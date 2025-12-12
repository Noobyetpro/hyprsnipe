use std::{
    fs,
    path::Path,
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use reqwest::{
    blocking::Client,
    header::{COOKIE, HeaderMap, HeaderValue, USER_AGENT},
};

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error:?}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    
    dotenvy::dotenv().ok();

    let base_url = std::env::var("BASE_URL")
        .context("BASE_URL environment variable is missing; set it in .env")?;
    let headers = build_headers()?;
    let mut client = build_client(&headers)?;
    let codes = read_codes(Path::new(".data.txt"))?;

    println!("Checking {} codes...", codes.len());

    let mut ok_200 = Vec::new();
    let mut bad_400 = Vec::new();
    let mut other = Vec::new();

    for code in codes {
        let code = code.trim();
        if code.is_empty() {
            continue;
        }

        let url = format!("{base_url}{code}");
        let start = Instant::now();
        let status = fetch_status_with_retry(&mut client, &url)?;
        let elapsed_ms = start.elapsed().as_millis();
        let status_code = status.as_u16();

        println!("{code}: {status_code} in {elapsed_ms} ms");

        match status_code {
            200 => ok_200.push(code.to_string()),
            400 => bad_400.push(code.to_string()),
            other_status => other.push((code.to_string(), other_status)),
        }

        thread::sleep(Duration::from_millis(1000));
    }

    write_results("results.txt", &ok_200, &bad_400, &other)?;

    println!(
        "Done. 200: {}, 400: {}, Other: {}. See results.txt.",
        ok_200.len(),
        bad_400.len(),
        other.len()
    );

    Ok(())
}

fn fetch_status_with_retry(client: &mut Client, url: &str) -> Result<reqwest::StatusCode> {
    let mut retry_count = 0u32;
    let retry_delay = Duration::from_millis(1000);

    loop {
        match client.get(url).send() {
            Ok(response) => {
                let status = response.status();
                if status.as_u16() == 429 {
                    retry_count += 1;
                    println!(
                        "Rate limited (429). Retrying in {} ms (attempt #{retry_count})...",
                        retry_delay.as_millis()
                    );
                    thread::sleep(retry_delay);
                    continue;
                }
                return Ok(status);
            }
            Err(err) if err.is_timeout() || err.is_connect() => {
                retry_count += 1;
                println!(
                    "Timeout/connect error. Retrying in {} ms (attempt #{retry_count})...",
                    retry_delay.as_millis()
                );
                thread::sleep(retry_delay);
                continue;
            }
            Err(err) => return Err(err).with_context(|| format!("failed to GET {url}")),
        }
    }
}

fn build_headers() -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();

    
    if let Ok(cookie) = std::env::var("COOKIE") {
        let cookie = cookie.trim();
        if !cookie.is_empty() {
            headers.insert(
                COOKIE,
                HeaderValue::from_str(cookie).context("invalid COOKIE header value")?,
            );
        }
    }

    
    let ua = std::env::var("USER_AGENT").unwrap_or_else(|_| "hyprsnipe-checker/0.1".to_string());
    headers.insert(
        USER_AGENT,
        HeaderValue::from_str(&ua).context("invalid USER_AGENT header value")?,
    );

    Ok(headers)
}

fn build_client(headers: &HeaderMap) -> Result<Client> {
    Client::builder()
        .default_headers(headers.clone())
        .timeout(Duration::from_secs(15))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .context("failed to build HTTP client")
}

fn read_codes(path: &Path) -> Result<Vec<String>> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;

    
    let list = contents
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    if list.is_empty() {
        anyhow::bail!("no codes found in {}", path.display());
    }

    Ok(list)
}

fn write_results(
    output_path: &str,
    ok_200: &[String],
    bad_400: &[String],
    other: &[(String, u16)],
) -> Result<()> {
    let mut output = String::new();

    output.push_str("200:\n");
    for code in ok_200 {
        output.push_str(code);
        output.push('\n');
    }
    output.push('\n');

    output.push_str("400:\n");
    for code in bad_400 {
        output.push_str(code);
        output.push('\n');
    }
    output.push('\n');

    output.push_str("OTHER:\n");
    for (code, status) in other {
        output.push_str(&format!("{status}: {code}\n"));
    }

    fs::write(output_path, output)
        .with_context(|| format!("failed to write results to {output_path}"))?;

    Ok(())
}

fn read_proxies(path: &Path) -> Result<Vec<String>> {
    let contents = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Ok(vec![]),
    };

    let list = contents
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    Ok(list)
}
