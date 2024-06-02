use crate::extract_required::{lz_compress, ParsedScript};
use crate::traversals;
use reqwest::blocking::Client;
use reqwest::header::HeaderValue;

static DOMAIN: &str = "cfschl.peet.ws";

struct Device<'a> {
    sec_ch_ua: &'a str,
    sec_ch_ua_mobile: &'a str,
    user_agent: &'a str,
    sec_ch_ua_platform: &'a str,
    language: &'a str,
}

impl Device<'_> {
    pub fn brave() -> Device<'static> {
        return Device {
            sec_ch_ua: "\"Chromium\";v=\"124\", \"Brave\";v=\"124\", \"Not-A.Brand\";v=\"99\"",
            sec_ch_ua_mobile: "?0",
            user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
            sec_ch_ua_platform: "\"macOS\"",
            language: "de-DE,de;q=0.7",
        };
    }
}

fn sh(s: &str) -> HeaderValue {
    return HeaderValue::from_str(s).unwrap();
}

pub struct SolvingSession<'a> {
    pub cnfg: traversals::config_builder::VMConfig,
    pub domain: String,
    pub debug: bool,

    client: Client,
    device: Device<'a>,
}

impl SolvingSession<'_> {
    pub fn new(domain: &str, debug: bool) -> SolvingSession {
        let tmp = Client::builder()
            .http1_title_case_headers()
            .brotli(true)
            .danger_accept_invalid_certs(true);
        let c: Client;
        if debug {
            println!("[DEBUG] Using debug proxy");
            c = tmp
                .proxy(reqwest::Proxy::all("http://localhost:8888").unwrap())
                .build()
                .unwrap();
        } else {
            c = tmp.build().unwrap()
        }

        return SolvingSession {
            domain: domain.to_owned(),
            cnfg: traversals::config_builder::VMConfig::default(),
            client: c,
            debug,
            device: Device::brave(),
        };
    }
    pub fn get_page(&self) -> Result<String, reqwest::Error> {
        let url = format!("https://{}/", DOMAIN);
        println!("GET {}", url);
        let resp = self
            .client
            .get(url)
            .header("upgrade-insecure-requests", "1")
            .header("user-agent", sh(self.device.user_agent))
            .header("accept", sh("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8"))
            .header("sec-ch-ua", sh(self.device.sec_ch_ua))
            .header("sec-ch-ua-mobile", sh(self.device.sec_ch_ua_mobile))
            .header("sec-ch-ua-platform", sh(self.device.sec_ch_ua_platform))
            .header("sec-ch-ua-platform-version", sh("\"14.3.0\""))
            .header("sec-ch-ua-model", sh("\"\""))
            .header("sec-gpc", sh("1"))
            .header("accept-language", sh(self.device.language))
            .header("sec-fetch-site", sh("none"))
            .header("sec-fetch-mode", sh("navigate"))
            .header("sec-fetch-user", sh("?1"))
            .header("sec-fetch-dest", sh("document"))
            .header("accept-encoding", sh("gzip, deflate, br, zstd"))
            .header("priority", sh("u=0, i"))
            .send();
        resp?.text()
    }

    pub fn get_script(&self) -> Result<String, reqwest::Error> {
        let url = format!(
            "https://{}/cdn-cgi/challenge-platform/h/{}/orchestrate/chl_page/v1?ray={}",
            self.domain, self.cnfg.chl_data.c_fpwv, self.cnfg.chl_data.c_ray
        );
        println!("GET {}", url);
        let referer = &format!("https://cfschl.peet.ws{}", self.cnfg.chl_data.fa);
        let resp = self
            .client
            .get(url)
            .header("sec-ch-ua", sh(self.device.sec_ch_ua))
            .header("sec-ch-ua-platform-version", sh("\"14.3.0\""))
            .header("sec-ch-ua-mobile", sh(self.device.sec_ch_ua_mobile))
            .header("sec-ch-ua-model", sh("\"\""))
            .header("user-agent", sh(self.device.user_agent))
            .header("sec-ch-ua-platform", sh(self.device.sec_ch_ua_platform))
            .header("accept", sh("*/*"))
            .header("sec-gpc", sh("1"))
            .header("accept-language", sh(self.device.language))
            .header("sec-fetch-site", sh("same-origin"))
            .header("sec-fetch-mode", sh("no-cors"))
            .header("sec-fetch-dest", sh("script"))
            .header("referer", sh(referer))
            .header("accept-encoding", sh("gzip, deflate, br, zstd"))
            .send();
        resp?.text()
    }

    pub fn submit_init(&self, script_data: &ParsedScript) -> Result<String, reqwest::Error> {
        let url = format!(
            "https://{}/cdn-cgi/challenge-platform/h/{}/flow/ov1/{}/{}/{}",
            self.domain,
            self.cnfg.chl_data.c_fpwv,
            script_data.path,
            self.cnfg.chl_data.c_ray,
            self.cnfg.chl_data.c_hash
        );

        let key: &[u8] = script_data.key.as_bytes();
        let payload = lz_compress(&self.cnfg.payloads.init, key);

        let c_ray = &self.cnfg.chl_data.c_ray;

        let body = format!("v_{}={}", c_ray, payload.replacen("+", "%2b", 1));

        // println!("POST {}", url);
        // println!("KEY: {}", script_data.key);
        // println!("{}", body);

        let resp = self
            .client
            .post(url)
            .header("content-length", sh(&format!("{}", body.len())))
            .header("sec-ch-ua", sh(self.device.sec_ch_ua))
            .header("sec-ch-ua-mobile", sh(self.device.sec_ch_ua_mobile))
            .header("user-agent", sh(self.device.user_agent))
            .header("content-type", sh("application/x-www-form-urlencoded"))
            .header("sec-ch-ua-platform-version", sh("\"14.3.0\""))
            .header("sec-ch-ua-model", sh("\"\""))
            .header("cf-challenge", sh(&self.cnfg.chl_data.c_hash))
            .header("sec-ch-ua-platform", sh(self.device.sec_ch_ua_platform))
            .header("accept", sh("*/*"))
            .header("sec-gpc", sh("1"))
            .header("accept-language", sh(self.device.language))
            .header("origin", sh("https://cfschl.peet.ws"))
            .header("sec-fetch-site", sh("same-origin"))
            .header("sec-fetch-mode", sh("cors"))
            .header("sec-fetch-dest", sh("empty"))
            .header("referer", sh("https://cfschl.peet.ws/"))
            .header("accept-encoding", sh("gzip, deflate, br, zstd"))
            .header("priority", sh("u=1, i"))
            .body(body)
            .send();
        resp?.text()
    }
}
