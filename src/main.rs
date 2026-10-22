//! Hifen 0.2: lugha ya Kiswahili yenye masharti na mizunguko.
//! Hakuna maktaba ya ziada: Rust standard library pekee.
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq)]
enum AinaToken {
    Jina(String), Andiko(String), Namba(String),
    Fungua, Funga, FunguaKizuizi, FungaKizuizi,
    Koma, Sawa, Linganisha(String), Jumlisha, Toa, Zidisha, Gawanya, Mwisho,
}
#[derive(Clone, Debug)]
struct Token { aina: AinaToken, mstari: usize }
fn kosa(mstari: usize, ujumbe: &str) -> String { format!("Mstari {}: {}", mstari, ujumbe) }

fn chambua(msimbo: &str) -> Result<Vec<Token>, String> {
    let mut out = Vec::new();
    let mut it = msimbo.chars().peekable();
    let mut mstari = 1;
    while let Some(&c) = it.peek() {
        if c == '\n' { it.next(); out.push(Token { aina: AinaToken::Mwisho, mstari }); mstari += 1; continue; }
        if c.is_whitespace() { it.next(); continue; }
        if c == '#' { while let Some(&x) = it.peek() { if x == '\n' { break; } it.next(); } continue; }
        if c == '/' {
            it.next();
            if it.peek() == Some(&'/') { while let Some(&x) = it.peek() { if x == '\n' { break; } it.next(); } continue; }
            out.push(Token { aina: AinaToken::Gawanya, mstari }); continue;
        }
        let aina = if c == '"' {
            it.next();
            let mut s = String::new(); let mut imefungwa = false;
            while let Some(x) = it.next() {
                if x == '"' { imefungwa = true; break; }
                if x == '\n' { return Err(kosa(mstari, "Maandishi hayajafungwa kwa alama za nukuu.")); }
                if x == '\\' {
                    match it.next() {
                        Some('n') => s.push('\n'), Some('t') => s.push('\t'),
                        Some('"') => s.push('"'), Some('\\') => s.push('\\'),
                        _ => return Err(kosa(mstari, "Alama ya kutoroka haijulikani.")),
                    }
                } else { s.push(x); }
            }
            if !imefungwa { return Err(kosa(mstari, "Maandishi hayajafungwa kwa alama za nukuu.")); }
            AinaToken::Andiko(s)
        } else if c.is_ascii_digit() {
            let mut s = String::new(); let mut nukta = 0;
            while let Some(&x) = it.peek() {
                if x.is_ascii_digit() || x == '.' {
                    if x == '.' { nukta += 1; }
                    s.push(x); it.next();
                } else { break; }
            }
            if nukta > 1 || s.ends_with('.') { return Err(kosa(mstari, "Namba si sahihi.")); }
            AinaToken::Namba(s)
        } else if c.is_alphabetic() || c == '_' {
            let mut s = String::new();
            while let Some(&x) = it.peek() {
                if x.is_alphanumeric() || x == '_' || x == '.' { s.push(x); it.next(); }
                else { break; }
            }
            AinaToken::Jina(s)
        } else {
            it.next();
            match c {
                '(' => AinaToken::Fungua,
                ')' => AinaToken::Funga,
                '{' => AinaToken::FunguaKizuizi,
                '}' => AinaToken::FungaKizuizi,
                ',' => AinaToken::Koma,
                '=' => {
                    if it.peek() == Some(&'=') { it.next(); AinaToken::Linganisha("==".into()) }
                    else { AinaToken::Sawa }
                }
                '!' => {
                    if it.peek() == Some(&'=') { it.next(); AinaToken::Linganisha("!=".into()) }
                    else { return Err(kosa(mstari, "Tumia '!=' badala ya '!'.")); }
                }
                '<' => {
                    if it.peek() == Some(&'=') { it.next(); AinaToken::Linganisha("<=".into()) }
                    else { AinaToken::Linganisha("<".into()) }
                }
                '>' => {
                    if it.peek() == Some(&'=') { it.next(); AinaToken::Linganisha(">=".into()) }
                    else { AinaToken::Linganisha(">".into()) }
                }
                '+' => AinaToken::Jumlisha,
                '-' => AinaToken::Toa,
                '*' => AinaToken::Zidisha,
                ';' => AinaToken::Mwisho,
                _ => return Err(kosa(mstari, &format!("Alama '{}' haitambuliki.", c))),
            }
        };
        out.push(Token { aina, mstari });
    }
    out.push(Token { aina: AinaToken::Mwisho, mstari });
    Ok(out)
}

#[derive(Clone, Debug)]
enum Usemi {
    Thamani(Thamani), Kigeu(String),
    Hesabu(Box<Usemi>, char, Box<Usemi>),
    Linganisha(Box<Usemi>, String, Box<Usemi>),
    Wito(String, Vec<Usemi>),
}
#[derive(Clone, Debug)]
enum Kauli {
    Tengeneza(Option<String>, String, Usemi),
    Badilisha(String, Usemi),
    Onyesha(Usemi),
    Tumia(Usemi),
    Kama(Usemi, Vec<Kauli>, Option<Vec<Kauli>>),
    Rudia(Usemi, Vec<Kauli>),
}
struct Mchambuzi { tokens: Vec<Token>, i: usize }
impl Mchambuzi {
    fn sasa(&self) -> &Token { &self.tokens[self.i.min(self.tokens.len()-1)] }
    fn kagua(&self, aina: &AinaToken) -> bool { &self.sasa().aina == aina }
    fn chukua(&mut self) -> Token { let t = self.sasa().clone(); if self.i < self.tokens.len()-1 { self.i += 1; } t }
    fn hitaji(&mut self, aina: AinaToken, maelezo: &str) -> Result<(), String> {
        if self.kagua(&aina) { self.chukua(); Ok(()) }
        else { Err(kosa(self.sasa().mstari, maelezo)) }
    }
    fn jina(&mut self) -> Result<String, String> {
        match self.chukua().aina {
            AinaToken::Jina(x) => Ok(x),
            _ => Err(kosa(self.tokens[self.i-1].mstari, "Jina linahitajika hapa.")),
        }
    }
    fn programu(&mut self) -> Result<Vec<Kauli>, String> {
        self.orodha_kauli(false)
    }

    // Huchambua amri hadi mwisho wa faili au hadi mabano ya kizuizi yafungwe.
    fn orodha_kauli(&mut self, ndani_ya_kizuizi: bool) -> Result<Vec<Kauli>, String> {
        let mut out = Vec::new();
        while self.i < self.tokens.len() - 1 {
            if self.kagua(&AinaToken::Mwisho) { self.chukua(); continue; }
            if self.kagua(&AinaToken::FungaKizuizi) {
                if ndani_ya_kizuizi { break; }
                return Err(kosa(self.sasa().mstari, "Kuna mabano '}' yasiyo na '{'."));
            }
            let kauli = self.kauli()?;
            out.push(kauli);
            if self.i < self.tokens.len() - 1
                && !self.kagua(&AinaToken::Mwisho)
                && !self.kagua(&AinaToken::FungaKizuizi)
            {
                return Err(kosa(self.sasa().mstari, "Maliza amri kwa mstari mpya au ';'."));
            }
        }
        Ok(out)
    }

    fn kauli(&mut self) -> Result<Kauli, String> {
        let n = self.sasa().mstari;
        let j = self.jina()?;

        if j == "kama" {
            self.hitaji(AinaToken::Fungua, "Tumia mabano '(' baada ya kama.")?;
            let sharti = self.usemi(0)?;
            self.hitaji(AinaToken::Funga, "Funga sharti kwa ')'.")?;
            let kama_kweli = self.kizuizi()?;

            // Ruhusu vinginevyo kuandikwa mstari unaofuata au mstari huohuo.
            let mut p = self.i;
            while p < self.tokens.len() && self.tokens[p].aina == AinaToken::Mwisho { p += 1; }
            let vinginevyo = if p < self.tokens.len()
                && matches!(&self.tokens[p].aina, AinaToken::Jina(x) if x.as_str() == "vinginevyo")
            {
                self.i = p + 1;
                Some(self.kizuizi()?)
            } else { None };
            return Ok(Kauli::Kama(sharti, kama_kweli, vinginevyo));
        }

        if j == "rudia" {
            self.hitaji(AinaToken::Fungua, "Tumia mabano '(' baada ya rudia.")?;
            let idadi = self.usemi(0)?;
            self.hitaji(AinaToken::Funga, "Funga idadi ya marudio kwa ')'.")?;
            let mwili = self.kizuizi()?;
            return Ok(Kauli::Rudia(idadi, mwili));
        }

        if j == "vinginevyo" {
            return Err(kosa(n, "vinginevyo lazima ifuate amri ya kama."));
        }

        if j == "badilisha" {
            let jina = self.jina()?;
            if jina.contains('.') {
                return Err(kosa(n, "Jina la kigeu lisiwe na nukta."));
            }
            self.hitaji(AinaToken::Sawa, "Weka '=' baada ya jina la kigeu.")?;
            return Ok(Kauli::Badilisha(jina, self.usemi(0)?));
        }

        let kauli = if j == "tengeneza" {
                let jina_la_kwanza = self.jina()?;
                let (aina, jina) = if ["sentensi", "namba", "desimali"].contains(&jina_la_kwanza.as_str()) {
                    (Some(jina_la_kwanza), self.jina()?)
                } else { (None, jina_la_kwanza) };
                if jina.contains('.') { return Err(kosa(n, "Jina la kigeu lisiwe na nukta.")); }
                self.hitaji(AinaToken::Sawa, "Weka '=' baada ya jina la kigeu.")?;
                Kauli::Tengeneza(aina, jina, self.usemi(0)?)
            } else {
                self.hitaji(AinaToken::Fungua, "Tumia mabano '()' baada ya amri.")?;
                let args = self.hoja()?;
                if j == "andika" {
                    if args.len() != 1 { return Err(kosa(n, "andika() inahitaji kitu kimoja.")); }
                    Kauli::Onyesha(args.into_iter().next().unwrap())
                } else { Kauli::Tumia(Usemi::Wito(j, args)) }
            };
        Ok(kauli)
    }

    fn kizuizi(&mut self) -> Result<Vec<Kauli>, String> {
        self.hitaji(AinaToken::FunguaKizuizi, "Weka '{' kuanza sehemu ya amri.")?;
        let amri = self.orodha_kauli(true)?;
        self.hitaji(AinaToken::FungaKizuizi, "Kizuizi hakijafungwa kwa '}'.")?;
        Ok(amri)
    }
    fn hoja(&mut self) -> Result<Vec<Usemi>, String> {
        let mut args = Vec::new();
        if self.kagua(&AinaToken::Funga) { self.chukua(); return Ok(args); }
        loop {
            args.push(self.usemi(0)?);
            if self.kagua(&AinaToken::Koma) { self.chukua(); }
            else { self.hitaji(AinaToken::Funga, "Funga mabano kwa ')'.")?; break; }
        }
        Ok(args)
    }
    fn usemi(&mut self, min: u8) -> Result<Usemi, String> {
        let token = self.chukua();
        let mut kushoto = match token.aina {
            AinaToken::Andiko(s) => Usemi::Thamani(Thamani::Sentensi(s)),
            AinaToken::Namba(s) => {
                if s.contains('.') {
                    Usemi::Thamani(Thamani::Desimali(s.parse().map_err(|_| kosa(token.mstari, "Desimali imezidi kiwango."))?))
                } else {
                    Usemi::Thamani(Thamani::Namba(s.parse().map_err(|_| kosa(token.mstari, "Namba imezidi kiwango."))?))
                }
            }
            AinaToken::Jina(j) => {
                if self.kagua(&AinaToken::Fungua) { self.chukua(); Usemi::Wito(j, self.hoja()?) }
                else { Usemi::Kigeu(j) }
            }
            AinaToken::Toa => Usemi::Hesabu(
                Box::new(Usemi::Thamani(Thamani::Namba(0))), '-', Box::new(self.usemi(5)?)),
            AinaToken::Fungua => { let x = self.usemi(0)?; self.hitaji(AinaToken::Funga, "Funga mabano kwa ')'.")?; x }
            _ => return Err(kosa(token.mstari, "Thamani au usemi unahitajika.")),
        };
        loop {
            let (nguvu, alama) = match &self.sasa().aina {
                AinaToken::Jumlisha => (1, "+".to_string()), AinaToken::Toa => (1, "-".to_string()),
                AinaToken::Zidisha => (3, "*".to_string()), AinaToken::Gawanya => (3, "/".to_string()),
                AinaToken::Linganisha(op) => (0, op.as_str().to_string()),
                _ => break,
            };
            if nguvu < min { break; }
            self.chukua();
            let kulia = self.usemi(nguvu + 1)?;
            if ["==", "!=", "<", "<=", ">", ">="].contains(&alama.as_str()) {
                kushoto = Usemi::Linganisha(Box::new(kushoto), alama, Box::new(kulia));
            } else {
                kushoto = Usemi::Hesabu(Box::new(kushoto), alama.chars().next().unwrap(), Box::new(kulia));
            }
        }
        Ok(kushoto)
    }
}

#[derive(Clone, Debug)]
struct Safu { mistari: usize, nguzo: usize, data: Vec<f64> }
#[derive(Clone, Debug)]
enum Thamani { Sentensi(String), Namba(i64), Desimali(f64), Mantiki(bool), Safu(Safu) }
impl std::fmt::Display for Thamani {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sentensi(x) => write!(f, "{}", x), Self::Namba(x) => write!(f, "{}", x),
            Self::Desimali(x) => write!(f, "{}", x),
            Self::Mantiki(x) => write!(f, "{}", if *x { "kweli" } else { "si kweli" }),
            Self::Safu(s) => {
                writeln!(f, "Safu ({} x {}):", s.mistari, s.nguzo)?;
                for r in 0..s.mistari { writeln!(f, "{:?}", &s.data[r*s.nguzo..(r+1)*s.nguzo])?; }
                Ok(())
            }
        }
    }
}
fn namba_ya(x: &Thamani) -> Result<f64, String> {
    match x { Thamani::Namba(n) => Ok(*n as f64), Thamani::Desimali(n) => Ok(*n),
        _ => Err("Tumia namba au desimali kwa hesabu.".into()) }
}
fn linganisha_thamani(a: &Thamani, alama: &str, b: &Thamani) -> Result<bool, String> {
    let kulinganisha_namba = match (a, b) {
        (Thamani::Namba(x), Thamani::Namba(y)) => Some((*x as f64, *y as f64)),
        (Thamani::Namba(x), Thamani::Desimali(y)) => Some((*x as f64, *y)),
        (Thamani::Desimali(x), Thamani::Namba(y)) => Some((*x, *y as f64)),
        (Thamani::Desimali(x), Thamani::Desimali(y)) => Some((*x, *y)),
        _ => None,
    };
    if let Some((x, y)) = kulinganisha_namba {
        return Ok(match alama {
            "==" => x == y, "!=" => x != y, "<" => x < y,
            "<=" => x <= y, ">" => x > y, ">=" => x >= y,
            _ => return Err("Alama ya kulinganisha haijulikani.".into()),
        });
    }
    match (a, b) {
        (Thamani::Sentensi(x), Thamani::Sentensi(y)) => Ok(match alama {
            "==" => x == y, "!=" => x != y, "<" => x < y,
            "<=" => x <= y, ">" => x > y, ">=" => x >= y,
            _ => return Err("Alama ya kulinganisha haijulikani.".into()),
        }),
        (Thamani::Mantiki(x), Thamani::Mantiki(y)) => match alama {
            "==" => Ok(x == y), "!=" => Ok(x != y),
            _ => Err("Kweli na si kweli hutumia '==' au '!=' tu.".into()),
        },
        _ => Err("Thamani hizi haziwezi kulinganishwa. Tumia aina zinazofanana.".into()),
    }
}
fn ukubwa(x: &Thamani) -> Result<usize, String> {
    match x { Thamani::Namba(n) if *n > 0 => usize::try_from(*n).map_err(|_| "Kipimo ni kikubwa mno.".into()),
        _ => Err("Kipimo cha safu lazima kiwe namba nzima kubwa kuliko sifuri.".into()) }
}
fn sentensi(x: &Thamani) -> Result<&str, String> {
    if let Thamani::Sentensi(s) = x { Ok(s) } else { Err("Hapa panahitaji sentensi.".into()) }
}
fn html_salama(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
      .replace('"', "&quot;").replace('\'', "&#39;")
}
fn rangi(s: &str) -> Result<String, String> {
    let r = match s { "nyeupe" => "#ffffff", "nyeusi" => "#000000",
        "bluu" => "#38bdf8", "kijani" => "#10b981", "nyekundu" => "#ef4444", _ => s };
    if r.len() != 7 || !r.starts_with('#') || !r[1..].chars().all(|x| x.is_ascii_hexdigit()) {
        return Err("Tumia rangi kama 'bluu' au '#112233'.".into());
    }
    Ok(r.to_string())
}
#[derive(Default)]
struct Tovuti { kichwa: String, maelezo: String, kitufe: String, nyuma: String }
impl Tovuti {
    fn mpya() -> Self { Self { kichwa: "Karibu Hifen".into(), maelezo: "Tovuti kwa Kiswahili".into(),
        kitufe: String::new(), nyuma: "#0f172a".into() } }
    fn html(&self) -> String {
        let kichwa = html_salama(&self.kichwa); let maelezo = html_salama(&self.maelezo);
        let kitufe = if self.kitufe.is_empty() { String::new() }
            else { format!("<button type=\"button\">{}</button>", html_salama(&self.kitufe)) };
        format!(r##"<!doctype html><html lang="sw"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>{kichwa}</title><style>
*{{box-sizing:border-box}}body{{margin:0;min-height:100vh;background:{nyuma};color:#f8fafc;font-family:system-ui,Arial,sans-serif;display:grid;place-items:center;padding:24px}}main{{width:min(100%,740px);padding:clamp(25px,6vw,70px);text-align:center;background:#ffffff0b;border:1px solid #ffffff25;border-radius:22px;box-shadow:0 22px 50px #0003}}h1{{font-size:clamp(2rem,6vw,4rem);line-height:1.15}}p{{color:#cbd5e1;line-height:1.8;font-size:1.1rem}}button{{background:#10b981;color:white;padding:14px 26px;border:0;border-radius:9px;font-weight:700;cursor:pointer}}button:focus-visible{{outline:3px solid #38bdf8;outline-offset:4px}}
</style></head><body><main><h1>{kichwa}</h1><p>{maelezo}</p>{kitufe}</main></body></html>"##,
            nyuma=self.nyuma)
    }
    fn washa(&self, port: u16) -> Result<(), String> {
        if port == 0 { return Err("Namba ya bandari haiwezi kuwa sifuri.".into()); }
        let anwani = format!("127.0.0.1:{}", port);
        let server = TcpListener::bind(&anwani).map_err(|e| format!("Seva haijawaka: {}", e))?;
        println!("Tovuti imewashwa: http://{}  (bonyeza Ctrl+C kusimamisha)", anwani);
        let body = self.html();
        for muunganisho in server.incoming() {
            let mut stream = match muunganisho { Ok(s) => s, Err(e) => { eprintln!("Muunganisho umekataa: {}", e); continue; } };
            let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));
            let _ = stream.set_write_timeout(Some(Duration::from_secs(3)));
            let mut buffer = [0u8; 4096];
            let n = match stream.read(&mut buffer) { Ok(n) if n > 0 => n, _ => continue };
            let request = String::from_utf8_lossy(&buffer[..n]);
            let (status, response_body) = if request.starts_with("GET / ") {
                ("200 OK", body.as_str())
            } else if request.starts_with("GET ") { ("404 Not Found", "Ukurasa haupo") }
            else { ("405 Method Not Allowed", "Njia hii hairuhusiwi") };
            let aina = if status == "200 OK" { "text/html" } else { "text/plain" };
            let kichwa = format!("HTTP/1.1 {}\r\nContent-Type: {}; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\nX-Content-Type-Options: nosniff\r\n\r\n", status, aina, response_body.len());
            let _ = stream.write_all(kichwa.as_bytes());
            let _ = stream.write_all(response_body.as_bytes());
        }
        Ok(())
    }
}
struct Mtendaji { vigeu: HashMap<String, Thamani>, tovuti: Tovuti }
impl Mtendaji {
    fn mpya() -> Self { Self { vigeu: HashMap::new(), tovuti: Tovuti::mpya() } }

    // Kila kizuizi hurithi vigeu vya nje. Mabadiliko ya vigeu vilivyokuwepo
    // yanaendelea baada ya kizuizi; vigeu vipya vilivyotengenezwa ndani hubaki vya ndani.
    fn endesha_kizuizi(&mut self, programu: &[Kauli]) -> Result<(), String> {
        let vigeu_vya_awali = self.vigeu.clone();
        match self.endesha(programu) {
            Ok(()) => {
                let vigeu_vya_mwisho = self.vigeu.clone();
                self.vigeu = vigeu_vya_awali;
                for jina in self.vigeu.keys().cloned().collect::<Vec<_>>() {
                    if let Some(thamani) = vigeu_vya_mwisho.get(&jina) {
                        self.vigeu.insert(jina, thamani.clone());
                    }
                }
                Ok(())
            }
            Err(kosa) => {
                self.vigeu = vigeu_vya_awali;
                Err(kosa)
            }
        }
    }

    fn tathmini(&mut self, e: &Usemi) -> Result<Thamani, String> {
        match e {
            Usemi::Thamani(x) => Ok(x.clone()),
            Usemi::Kigeu(j) => self.vigeu.get(j).cloned().ok_or_else(|| format!("Kigeu '{}' hakijatengenezwa.", j)),
            Usemi::Hesabu(a, op, b) => {
                let x = self.tathmini(a)?; let y = self.tathmini(b)?;
                if let (Thamani::Sentensi(a), Thamani::Sentensi(b)) = (&x, &y) {
                    if *op == '+' { return Ok(Thamani::Sentensi(format!("{}{}", a, b))); }
                }
                if let (Thamani::Namba(a), Thamani::Namba(b)) = (&x, &y) {
                    let z = match *op { '+' => a.checked_add(*b), '-' => a.checked_sub(*b),
                        '*' => a.checked_mul(*b), _ => None };
                    if let Some(v) = z { return Ok(Thamani::Namba(v)); }
                    if *op != '/' { return Err("Namba imezidi kiwango cha hesabu.".into()); }
                }
                let a = namba_ya(&x)?; let b = namba_ya(&y)?;
                if *op == '/' && b == 0.0 { return Err("Huwezi kugawanya kwa sifuri.".into()); }
                let v = match *op { '+' => a+b, '-' => a-b, '*' => a*b, '/' => a/b, _ => unreachable!() };
                if !v.is_finite() { return Err("Jibu la hesabu limezidi kiwango.".into()); }
                Ok(Thamani::Desimali(v))
            }
            Usemi::Linganisha(a, op, b) => {
                let x = self.tathmini(a)?;
                let y = self.tathmini(b)?;
                Ok(Thamani::Mantiki(linganisha_thamani(&x, op, &y)?))
            }
            Usemi::Wito(jina, hoja) => {
                let args: Vec<Thamani> = hoja.iter().map(|a| self.tathmini(a)).collect::<Result<_,_>>()?;
                match jina.as_str() {
                    "AkiliMnemba.tengeneza_safu" => {
                        if args.len()!=2 { return Err("tengeneza_safu inahitaji vipimo viwili.".into()); }
                        let m = ukubwa(&args[0])?; let n = ukubwa(&args[1])?;
                        let jumla = m.checked_mul(n).ok_or("Vipimo ni vikubwa mno.")?;
                        if jumla > 1_000_000 { return Err("Safu imezidi elementi 1,000,000.".into()); }
                        Ok(Thamani::Safu(Safu { mistari:m, nguzo:n, data:vec![0.0;jumla] }))
                    }
                    "AkiliMnemba.jumlisha_safu" => {
                        if args.len()!=2 { return Err("jumlisha_safu inahitaji safu mbili.".into()); }
                        if let (Thamani::Safu(a), Thamani::Safu(b)) = (&args[0], &args[1]) {
                            if a.mistari!=b.mistari || a.nguzo!=b.nguzo { return Err("Safu hazina vipimo sawa.".into()); }
                            Ok(Thamani::Safu(Safu { mistari:a.mistari, nguzo:a.nguzo,
                                data:a.data.iter().zip(&b.data).map(|(x,y)| x+y).collect() }))
                        } else { Err("Tumia safu mbili.".into()) }
                    }
                    _ => Err(format!("Kazi '{}' haijatambulika au hairudishi thamani.", jina)),
                }
            }
        }
    }
    fn endesha(&mut self, programu: &[Kauli]) -> Result<(), String> {
        for k in programu {
            match k {
                Kauli::Tengeneza(aina, jina, e) => {
                    let v = self.tathmini(e)?;
                    if let Some(aina) = aina {
                        let sahihi = matches!((aina.as_str(), &v),
                            ("namba", Thamani::Namba(_)) | ("desimali", Thamani::Desimali(_)) |
                            ("sentensi", Thamani::Sentensi(_)));
                        if !sahihi { return Err(format!("Kigeu '{}' kinahitaji aina '{}'.", jina, aina)); }
                    }
                    if self.vigeu.contains_key(jina) { return Err(format!("Kigeu '{}' tayari kipo.", jina)); }
                    self.vigeu.insert(jina.clone(), v);
                }
                Kauli::Badilisha(jina, e) => {
                    let mpya = self.tathmini(e)?;
                    let ya_awali = self.vigeu.get(jina)
                        .ok_or_else(|| format!("Kigeu '{}' hakijatengenezwa. Tumia 'tengeneza' kwanza.", jina))?;
                    let aina_sawa = matches!((ya_awali, &mpya),
                        (Thamani::Sentensi(_), Thamani::Sentensi(_)) |
                        (Thamani::Namba(_), Thamani::Namba(_)) |
                        (Thamani::Desimali(_), Thamani::Desimali(_)) |
                        (Thamani::Mantiki(_), Thamani::Mantiki(_)));
                    if !aina_sawa {
                        return Err(format!("Aina ya kigeu '{}' haiwezi kubadilishwa. Tumia aina ileile ya thamani.", jina));
                    }
                    self.vigeu.insert(jina.clone(), mpya);
                }
                Kauli::Onyesha(e) => println!("{}", self.tathmini(e)?),
                Kauli::Kama(sharti, kama_kweli, vinginevyo) => {
                    let matokeo = self.tathmini(sharti)?;
                    let chaguo = match matokeo {
                        Thamani::Mantiki(true) => Some(kama_kweli),
                        Thamani::Mantiki(false) => vinginevyo.as_ref(),
                        _ => return Err("Sharti la 'kama' lazima liwe la kulinganisha, mfano umri >= 18.".into()),
                    };
                    if let Some(mwili) = chaguo {
                        self.endesha_kizuizi(mwili)?;
                    }
                }
                Kauli::Rudia(idadi, mwili) => {
                    let thamani = self.tathmini(idadi)?;
                    let mara = match thamani {
                        Thamani::Namba(n) if n >= 0 => usize::try_from(n)
                            .map_err(|_| "Idadi ya marudio ni kubwa mno.")?,
                        _ => return Err("Idadi ya rudia lazima iwe namba nzima isiyo hasi.".into()),
                    };
                    if mara > 1_000_000 {
                        return Err("Rudia imezidi kiwango cha mara 1,000,000.".into());
                    }
                    for _ in 0..mara {
                        self.endesha_kizuizi(mwili)?;
                    }
                }
                Kauli::Tumia(Usemi::Wito(j, args)) => {
                    let a: Vec<Thamani> = args.iter().map(|x| self.tathmini(x)).collect::<Result<_,_>>()?;
                    match j.as_str() {
                        "UkurasaMtandao.weka_kichwa" | "UkurasaMtandao.weka_maelezo" |
                        "UkurasaMtandao.weka_kitufe" | "UkurasaMtandao.weka_rangi" => {
                            if a.len()!=1 { return Err(format!("{} inahitaji sentensi moja.", j)); }
                            let s = sentensi(&a[0])?.to_string();
                            match j.as_str() {
                                "UkurasaMtandao.weka_kichwa" => self.tovuti.kichwa=s,
                                "UkurasaMtandao.weka_maelezo" => self.tovuti.maelezo=s,
                                "UkurasaMtandao.weka_kitufe" => self.tovuti.kitufe=s,
                                _ => self.tovuti.nyuma=rangi(&s)?,
                            }
                        }
                        "UkurasaMtandao.washa_server" => {
                            if a.len()!=1 { return Err("washa_server inahitaji namba moja.".into()); }
                            let port = match &a[0] { Thamani::Namba(n) => u16::try_from(*n)
                                .map_err(|_| "Bandari iwe kati ya 1 na 65535.")?,
                                _ => return Err("Bandari lazima iwe namba nzima.".into()) };
                            self.tovuti.washa(port)?;
                        }
                        _ => return Err(format!("Amri '{}' haijatambulika.", j)),
                    }
                }
                _ => return Err("Amri haitambuliki.".into()),
            }
        }
        Ok(())
    }
}
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len()!=3 || args[1]!="endesha" {
        eprintln!("Matumizi: hifen endesha faili.hifen");
        std::process::exit(2);
    }
    let matokeo = (|| -> Result<(),String> {
        let msimbo = fs::read_to_string(&args[2]).map_err(|e| format!("Faili halijasomeka: {}",e))?;
        let tokens = chambua(&msimbo)?;
        let programu = Mchambuzi { tokens, i:0 }.programu()?;
        Mtendaji::mpya().endesha(&programu)
    })();
    if let Err(e) = matokeo { eprintln!("KOSA LA HIFEN: {}", e); std::process::exit(1); }
}
