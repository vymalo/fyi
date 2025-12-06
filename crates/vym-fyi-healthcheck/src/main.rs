use std::env;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

struct Config {
    host: String,
    port: u16,
    path: String,
    timeout: Duration,
}

impl Config {
    fn from_env_and_args() -> Result<Self, String> {
        let mut host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let mut port = env::var("PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3000);
        let mut path = env::var("HEALTH_PATH").unwrap_or_else(|_| "/health".to_string());
        let mut timeout_ms = env::var("HEALTH_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3000);

        if let Ok(scheme) = env::var("HEALTH_SCHEME")
            && scheme.to_lowercase() != "http"
        {
            return Err("only http health checks are supported".to_string());
        }

        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--host" => host = args.next().ok_or("--host requires a value")?,
                "--port" => {
                    let raw = args.next().ok_or("--port requires a value")?;
                    port = raw.parse().map_err(|_| "--port must be a number")?;
                }
                "--path" => path = args.next().ok_or("--path requires a value")?,
                "--timeout-ms" => {
                    let raw = args.next().ok_or("--timeout-ms requires a value")?;
                    timeout_ms = raw.parse().map_err(|_| "--timeout-ms must be a number")?;
                }
                "-h" | "--help" => {
                    print_usage();
                    std::process::exit(0);
                }
                other => return Err(format!("unknown argument: {}", other)),
            }
        }

        if !path.starts_with('/') {
            path = format!("/{}", path);
        }

        Ok(Self {
            host,
            port,
            path,
            timeout: Duration::from_millis(timeout_ms.max(1)),
        })
    }
}

fn main() {
    let config = match Config::from_env_and_args() {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("healthcheck config error: {}", err);
            std::process::exit(1);
        }
    };

    if let Err(err) = run_check(&config) {
        eprintln!("healthcheck failed: {}", err);
        std::process::exit(1);
    }

    println!(
        "healthcheck OK: http://{}:{}{}",
        config.host, config.port, config.path
    );
}

fn run_check(cfg: &Config) -> Result<(), String> {
    let mut addrs = format!("{}:{}", cfg.host, cfg.port)
        .to_socket_addrs()
        .map_err(|e| format!("resolve error: {}", e))?;
    let addr = addrs
        .next()
        .ok_or_else(|| "no resolved addresses".to_string())?;

    let mut stream = TcpStream::connect_timeout(&addr, cfg.timeout)
        .map_err(|e| format!("connect error: {}", e))?;
    stream
        .set_read_timeout(Some(cfg.timeout))
        .map_err(|e| format!("read timeout set failed: {}", e))?;
    stream
        .set_write_timeout(Some(cfg.timeout))
        .map_err(|e| format!("write timeout set failed: {}", e))?;

    let host_header = format!("{}:{}", cfg.host, cfg.port);
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: vym-fyi-healthcheck\r\nConnection: close\r\n\r\n",
        cfg.path, host_header
    );

    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("write error: {}", e))?;

    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .map_err(|e| format!("read error: {}", e))?;

    let response_text = String::from_utf8_lossy(&response);
    let status_line = response_text
        .lines()
        .next()
        .ok_or_else(|| "empty response".to_string())?;

    let status = status_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| "malformed status line".to_string())?;
    let code: u16 = status
        .parse()
        .map_err(|_| format!("invalid status code: {}", status))?;

    if (200..=299).contains(&code) {
        Ok(())
    } else {
        Err(format!("unexpected status: {}", code))
    }
}

fn print_usage() {
    eprintln!("Usage: healthcheck [--host HOST] [--port PORT] [--path PATH] [--timeout-ms MS]");
    eprintln!("Defaults: HOST=127.0.0.1, PORT=3000, PATH=/health, TIMEOUT=3000ms");
}
