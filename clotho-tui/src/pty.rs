use std::io::{Read, Write};
use std::sync::{Arc, Mutex, RwLock};

use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use tokio::sync::mpsc;

/// Manages the embedded PTY running the claude CLI.
pub struct PtyHandle {
    /// VT100 parser — shared between the reader thread and the render loop.
    pub parser: Arc<RwLock<vt100::Parser>>,
    /// Send bytes to the PTY's stdin.
    pub input_tx: mpsc::UnboundedSender<Vec<u8>>,
    /// PTY master — kept alive for resizing.
    master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
    /// Last known size.
    last_rows: u16,
    last_cols: u16,
}

impl PtyHandle {
    /// Spawn a PTY running the given command with the given size.
    pub fn spawn(
        cmd: &str,
        args: &[String],
        working_dir: &std::path::Path,
        rows: u16,
        cols: u16,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        // Build the command
        let mut command = CommandBuilder::new(cmd);
        command.args(args);
        command.cwd(working_dir);

        // Spawn the child process on the slave side
        let _child = pair.slave.spawn_command(command)?;
        // Drop the slave — we interact through the master
        drop(pair.slave);

        let parser = Arc::new(RwLock::new(vt100::Parser::new(rows, cols, 1000)));

        // Reader thread: reads PTY output and feeds it to the vt100 parser
        let reader_parser = Arc::clone(&parser);
        let mut reader = pair.master.try_clone_reader()?;
        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break, // EOF — child exited
                    Ok(n) => {
                        if let Ok(mut p) = reader_parser.write() {
                            p.process(&buf[..n]);
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        // Writer channel: receives bytes from the main loop and writes to PTY stdin
        let (input_tx, mut input_rx) = mpsc::unbounded_channel::<Vec<u8>>();
        let mut writer = pair.master.take_writer()?;
        std::thread::spawn(move || {
            while let Some(bytes) = input_rx.blocking_recv() {
                if writer.write_all(&bytes).is_err() {
                    break;
                }
                let _ = writer.flush();
            }
        });

        let master = Arc::new(Mutex::new(pair.master));

        Ok(Self {
            parser,
            input_tx,
            master,
            last_rows: rows,
            last_cols: cols,
        })
    }

    /// Send raw bytes to the PTY (keyboard input).
    pub fn send_input(&self, bytes: Vec<u8>) {
        let _ = self.input_tx.send(bytes);
    }

    /// Resize the PTY if the size has changed.
    pub fn resize(&mut self, rows: u16, cols: u16) {
        if rows == self.last_rows && cols == self.last_cols {
            return;
        }
        self.last_rows = rows;
        self.last_cols = cols;

        // Resize the actual PTY
        if let Ok(master) = self.master.lock() {
            let _ = master.resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            });
        }

        // Resize the vt100 parser
        if let Ok(mut p) = self.parser.write() {
            p.set_size(rows, cols);
        }
    }
}
