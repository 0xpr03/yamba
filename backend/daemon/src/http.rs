/*
 *  This file is part of yamba.
 *
 *  yamba is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  yamba is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with yamba.  If not, see <https://www.gnu.org/licenses/>.
 */
use failure::Fallible;
use reqwest::header::HeaderMap;
use reqwest::header::{
    ACCEPT, ACCEPT_ENCODING, CONNECTION, CONTENT_ENCODING, LOCATION, USER_AGENT,
};
use reqwest::{Client, Response};
use std::fs::File;
use std::path::Path;
use std::time::Duration;

use crate::SETTINGS;

/// Http request abstraction for blocking http requests

/// Header type for get requests
pub enum HeaderType {
    /// Html browser request
    Html,
    /// Ajax js request
    Ajax,
}

/// Does a text get request under the provided url & header
pub fn get_text(url: &str, htype: HeaderType) -> Fallible<String> {
    Ok(get_raw(url, htype)?.text()?)
}

/// Download into file, return of length in bytes
pub fn get_file(url: &str, path: &Path) -> Fallible<u64> {
    trace!("Download path: {:?}", path);
    let mut response = get_raw(url, HeaderType::Html)?;
    let mut file = File::create(path)?;
    Ok(response.copy_to(&mut file)?)
}

/// Does a raw get request under the provided url & header
fn get_raw(url: &str, htype: HeaderType) -> Fallible<Response> {
    trace!("Starting request {}", url);

    let client = Client::builder()
        .gzip(true)
        .timeout(Duration::from_secs(10))
        .build()?;
    let builder = client.get(url);
    let res = builder.headers(header(htype)).send()?;

    debug!("Response header: {:?}", res.headers());
    debug!("Response status: {:?}", res.status());
    debug!("Final URL: {:?}", res.headers().get(LOCATION));
    trace!("DEV header: {:?}", res.headers().get(CONTENT_ENCODING));
    Ok(res)
}

/// Construct a header
/// This function does not check for errors and is
/// verified by the tests
fn header(htype: HeaderType) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT_ENCODING, "gzip, deflate, br".parse().unwrap());

    match htype {
        HeaderType::Html => {
            headers.insert(
                ACCEPT,
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
                    .parse()
                    .unwrap(),
            );
        }
        HeaderType::Ajax => {
            headers.insert(
                ACCEPT,
                "application/json, text/javascript, */*; q=0.01"
                    .parse()
                    .unwrap(),
            );
        }
    }
    headers.insert(CONNECTION, "close".parse().unwrap());
    headers.insert(USER_AGENT, SETTINGS.main.user_agent.parse().unwrap());

    trace!("Generated headers: {:?}", headers);
    headers
}

#[cfg(test)]
mod test {
    use super::header;
    use super::*;

    /// Test header creation
    #[test]
    fn header_test() {
        let _ = header(HeaderType::Html);
        let _ = header(HeaderType::Ajax);
    }

    /// Test a html get request
    #[test]
    fn get_html_gzipped() {
        let b_html: String = get_text("https://httpbin.org/gzip", HeaderType::Html).unwrap();
        assert!(true, b_html.contains(r#""gzipped": true"#));
    }

    /// Test a ajax json get request
    #[test]
    fn get_ajax() {
        let b_ajax: String = get_text("https://httpbin.org/user-agent", HeaderType::Ajax).unwrap();
        assert!(b_ajax.contains(&SETTINGS.main.user_agent));
    }
}
