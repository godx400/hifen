//! Hifen 0.4: lugha ya Kiswahili yenye mantiki na muundo wa tovuti wa ndani.
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
    Mantiki(Box<Usemi>, String, Box<Usemi>),
    MantikiKinyume(Box<Usemi>),
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
    Tovuti(Tovuti),
    TengenezaKazi { jina: String, vigeu: Vec<String>, mwili: Vec<Kauli> },
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

        // Tovuti nzima hufafanuliwa kwenye faili moja la .hifen.
        if j == "tovuti" {
            return self.tovuti_mpya(n).map(Kauli::Tovuti);
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

        if j == "tengeneza" {
            // Kazi zinazoundwa na mtumiaji: tengeneza kazi salimia(jina) { ... }
            if matches!(&self.sasa().aina, AinaToken::Jina(x) if x == "kazi") {
                self.chukua();
                let jina_kazi = self.jina()?;
                if jina_kazi.contains('.') {
                    return Err(kosa(n, "Jina la kazi liwe neno moja bila nukta."));
                }
                if ["tengeneza", "andika", "kama", "vinginevyo", "rudia", "tovuti", "seva"].contains(&jina_kazi.as_str()) {
                    return Err(kosa(n, "Jina hili limehifadhiwa na Hifen; chagua jina jingine."));
                }
                self.hitaji(AinaToken::Fungua, "Weka '(' baada ya jina la kazi.")?;
                let mut vigeu = Vec::new();
                if !self.kagua(&AinaToken::Funga) {
                    loop {
                        let jina_kigeu = self.jina()?;
                        if jina_kigeu.contains('.') {
                            return Err(kosa(n, "Jina la kigeu cha kazi lisiwe na nukta."));
                        }
                        if vigeu.contains(&jina_kigeu) {
                            return Err(kosa(n, "Kigeu kimoja kimerudiwa kwenye vigeu vya kazi."));
                        }
                        vigeu.push(jina_kigeu);
                        if self.kagua(&AinaToken::Koma) { self.chukua(); }
                        else { break; }
                    }
                }
                self.hitaji(AinaToken::Funga, "Funga orodha ya vigeu vya kazi kwa ')'.")?;
                let mwili = self.kizuizi()?;
                return Ok(Kauli::TengenezaKazi { jina: jina_kazi, vigeu, mwili });
            }

            let jina_la_kwanza = self.jina()?;
            let (aina, jina) = if ["sentensi", "namba", "desimali", "mantiki"].contains(&jina_la_kwanza.as_str()) {
                (Some(jina_la_kwanza), self.jina()?)
            } else { (None, jina_la_kwanza) };
            if jina.contains('.') { return Err(kosa(n, "Jina la kigeu lisiwe na nukta.")); }
            self.hitaji(AinaToken::Sawa, "Weka '=' baada ya jina la kigeu.")?;
            return Ok(Kauli::Tengeneza(aina, jina, self.usemi(0)?));
        }

        self.hitaji(AinaToken::Fungua, "Tumia mabano '()' baada ya amri.")?;
        let args = self.hoja()?;
        if j == "andika" {
            if args.len() != 1 { return Err(kosa(n, "andika() inahitaji kitu kimoja.")); }
            Ok(Kauli::Onyesha(args.into_iter().next().unwrap()))
        } else {
            Ok(Kauli::Tumia(Usemi::Wito(j, args)))
        }
    }

    fn andiko_linalofuata(&mut self, maelezo: &str) -> Result<String, String> {
        let t = self.chukua();
        match t.aina {
            AinaToken::Andiko(s) => Ok(s),
            _ => Err(kosa(t.mstari, maelezo)),
        }
    }

    fn namba_linalofuata(&mut self, maelezo: &str) -> Result<u32, String> {
        let t = self.chukua();
        match t.aina {
            AinaToken::Namba(s) if !s.contains('.') => s.parse::<u32>()
                .map_err(|_| kosa(t.mstari, "Namba hii ni kubwa mno.")),
            _ => Err(kosa(t.mstari, maelezo)),
        }
    }

    fn mantiki_linalofuata(&mut self, maelezo: &str) -> Result<bool, String> {
        let t = self.chukua();
        match t.aina {
            AinaToken::Jina(s) if s == "kweli" => Ok(true),
            AinaToken::Jina(s) if s == "sikweli" || s == "si_kweli" => Ok(false),
            _ => Err(kosa(t.mstari, maelezo)),
        }
    }

    fn ruka_mistari_mipya(&mut self) {
        while self.i < self.tokens.len() - 1 && self.kagua(&AinaToken::Mwisho) {
            self.chukua();
        }
    }

    fn tovuti_mpya(&mut self, mstari: usize) -> Result<Tovuti, String> {
        let jina = self.andiko_linalofuata("Tumia jina la tovuti ndani ya alama za nukuu, mfano tovuti \"Hifen\" {")?;
        self.hitaji(AinaToken::FunguaKizuizi, "Weka '{' baada ya jina la tovuti.")?;
        let mut tovuti = Tovuti::mpya();
        tovuti.kichwa_kivinjari = jina;
        let mut ukurasa_umewekwa = false;
        let mut mtindo_umewekwa = false;
        let mut seva_imewekwa = false;

        loop {
            self.ruka_mistari_mipya();
            if self.kagua(&AinaToken::FungaKizuizi) {
                self.chukua();
                break;
            }
            if self.i >= self.tokens.len() - 1 {
                return Err(kosa(mstari, "Sehemu ya tovuti haijafungwa kwa '}'."));
            }
            let t = self.chukua();
            let amri = match t.aina {
                AinaToken::Jina(j) => j,
                _ => return Err(kosa(t.mstari, "Weka amri ya tovuti, kama ukurasa, mitindo au seva.")),
            };
            match amri.as_str() {
                "kichwa_cha_kivinjari" => {
                    tovuti.kichwa_kivinjari = self.andiko_linalofuata("Weka jina la kivinjari ndani ya alama za nukuu.")?;
                }
                "ukurasa" => {
                    if ukurasa_umewekwa {
                        return Err(kosa(t.mstari, "Tovuti inaweza kuwa na sehemu moja ya 'ukurasa' kwa sasa."));
                    }
                    self.hitaji(AinaToken::FunguaKizuizi, "Weka '{' baada ya ukurasa.")?;
                    tovuti.maudhui = self.vipengele_ukurasa()?;
                    self.hitaji(AinaToken::FungaKizuizi, "Funga ukurasa kwa '}'.")?;
                    ukurasa_umewekwa = true;
                }
                "mitindo" => {
                    if mtindo_umewekwa {
                        return Err(kosa(t.mstari, "Tovuti inaweza kuwa na sehemu moja ya 'mitindo'."));
                    }
                    self.hitaji(AinaToken::FunguaKizuizi, "Weka '{' baada ya mitindo.")?;
                    self.mitindo_wa_tovuti(&mut tovuti, t.mstari)?;
                    mtindo_umewekwa = true;
                }
                "seva" => {
                    let n = self.namba_linalofuata("Weka namba ya seva, mfano seva 8080.")?;
                    if n == 0 || n > u16::MAX as u32 {
                        return Err(kosa(t.mstari, "Namba ya seva lazima iwe kati ya 1 na 65535."));
                    }
                    tovuti.bandari = n as u16;
                    seva_imewekwa = true;
                }
                _ => return Err(kosa(t.mstari, &format!("Amri ya tovuti '{}' haitambuliki.", amri))),
            }
        }
        if !ukurasa_umewekwa {
            return Err(kosa(mstari, "Ongeza sehemu ya 'ukurasa { ... }' ndani ya tovuti."));
        }
        if !seva_imewekwa {
            // Chaguo la kawaida kwa tovuti ya majaribio ya ndani.
            tovuti.bandari = 8080;
        }
        tovuti.thibitisha_mitindo().map_err(|e| kosa(mstari, &e))?;
        Ok(tovuti)
    }

    // Husoma kichwa, aya, kitufe na sehemu zenye vipengele vingine ndani yake.
    // Inarudi pale inapokutana na '}' ya sehemu ya sasa; caller hufunga mabano hayo.
    fn vipengele_ukurasa(&mut self) -> Result<Vec<Kipengele>, String> {
        let mut vipengele = Vec::new();
        loop {
            self.ruka_mistari_mipya();
            if self.kagua(&AinaToken::FungaKizuizi) { break; }
            if self.i >= self.tokens.len() - 1 {
                return Err(kosa(self.sasa().mstari, "Sehemu ya ukurasa haijafungwa kwa '}'."));
            }
            let t = self.chukua();
            let amri = match t.aina {
                AinaToken::Jina(j) => j,
                _ => return Err(kosa(t.mstari, "Tumia kichwa, maelezo, kitufe au sehemu ndani ya ukurasa.")),
            };
            match amri.as_str() {
                "kichwa" => vipengele.push(Kipengele::Kichwa(self.andiko_linalofuata("Kichwa kinahitaji maandishi ndani ya nukuu.")?)),
                "kichwa_dogo" => vipengele.push(Kipengele::KichwaDogo(self.andiko_linalofuata("Kichwa kidogo kinahitaji maandishi ndani ya nukuu.")?)),
                "maelezo" | "aya" => vipengele.push(Kipengele::Aya(self.andiko_linalofuata("Maelezo yanahitaji maandishi ndani ya nukuu.")?)),
                "kitufe" => vipengele.push(Kipengele::Kitufe(self.andiko_linalofuata("Kitufe kinahitaji jina ndani ya nukuu.")?)),
                "sehemu" | "kisanduku" => {
                    let jina = self.andiko_linalofuata("Sehemu inahitaji jina ndani ya nukuu.")?;
                    self.hitaji(AinaToken::FunguaKizuizi, "Weka '{' baada ya jina la sehemu.")?;
                    let watoto = self.vipengele_ukurasa()?;
                    self.hitaji(AinaToken::FungaKizuizi, "Funga sehemu kwa '}'.")?;
                    vipengele.push(Kipengele::Sehemu { jina, watoto });
                }
                _ => return Err(kosa(t.mstari, &format!("Kipengele cha ukurasa '{}' hakitambuliki.", amri))),
            }
        }
        Ok(vipengele)
    }

    fn mitindo_wa_tovuti(&mut self, tovuti: &mut Tovuti, mstari: usize) -> Result<(), String> {
        loop {
            self.ruka_mistari_mipya();
            if self.kagua(&AinaToken::FungaKizuizi) { self.chukua(); break; }
            if self.i >= self.tokens.len() - 1 {
                return Err(kosa(mstari, "Sehemu ya mitindo haijafungwa kwa '}'."));
            }
            let t = self.chukua();
            let jina = match t.aina {
                AinaToken::Jina(j) => j,
                _ => return Err(kosa(t.mstari, "Weka jina la mtindo, mfano fonti au padding.")),
            };
            match jina.as_str() {
                "fonti" => tovuti.fonti = self.andiko_linalofuata("Weka fonti ndani ya alama za nukuu.")?,
                "rangi_ya_nyuma" => tovuti.nyuma = rangi(&self.andiko_linalofuata("Weka rangi ndani ya alama za nukuu.")?)?,
                "rangi_ya_maandishi" => tovuti.rangi_maandishi = rangi(&self.andiko_linalofuata("Weka rangi ya maandishi ndani ya nukuu.")?)?,
                "rangi_ya_msingi" => tovuti.rangi_msingi = rangi(&self.andiko_linalofuata("Weka rangi ya msingi ndani ya nukuu.")?)?,
                "padding" => tovuti.padding = self.namba_linalofuata("padding inahitaji namba nzima.")?,
                "pengo" => tovuti.pengo = self.namba_linalofuata("pengo linahitaji namba nzima.")?,
                "upana" => tovuti.upana = self.namba_linalofuata("upana unahitaji namba nzima.")?,
                "pembe" => tovuti.pembe = self.namba_linalofuata("pembe inahitaji namba nzima.")?,
                "ukubwa_wa_kichwa" => tovuti.ukubwa_kichwa = self.namba_linalofuata("ukubwa_wa_kichwa unahitaji namba nzima.")?,
                "mpangilio" => tovuti.mpangilio = self.andiko_linalofuata("Tumia 'wima' au 'mlalo' ndani ya nukuu.")?,
                "panga" => tovuti.panga = self.andiko_linalofuata("Tumia 'katikati', 'anza' au 'mwisho' ndani ya nukuu.")?,
                "kivuli" => tovuti.kivuli = self.mantiki_linalofuata("kivuli kinahitaji kweli au sikweli.")?,
                _ => return Err(kosa(t.mstari, &format!("Mtindo '{}' hautambuliki.", jina))),
            }
        }
        Ok(())
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
            AinaToken::Jina(j) => match j.as_str() {
                "kweli" => Usemi::Thamani(Thamani::Mantiki(true)),
                "sikweli" | "si_kweli" => Usemi::Thamani(Thamani::Mantiki(false)),
                "si" => Usemi::MantikiKinyume(Box::new(self.usemi(2)?)),
                _ => {
                    if self.kagua(&AinaToken::Fungua) { self.chukua(); Usemi::Wito(j, self.hoja()?) }
                    else { Usemi::Kigeu(j) }
                }
            },
            AinaToken::Toa => Usemi::Hesabu(
                Box::new(Usemi::Thamani(Thamani::Namba(0))), '-', Box::new(self.usemi(5)?)),
            AinaToken::Fungua => { let x = self.usemi(0)?; self.hitaji(AinaToken::Funga, "Funga mabano kwa ')'.")?; x }
            _ => return Err(kosa(token.mstari, "Thamani au usemi unahitajika.")),
        };
        loop {
            let (nguvu, alama) = match &self.sasa().aina {
                AinaToken::Jina(op) if op == "au" => (0, "au".to_string()),
                AinaToken::Jina(op) if op == "na" => (1, "na".to_string()),
                AinaToken::Linganisha(op) => (2, op.as_str().to_string()),
                AinaToken::Jumlisha => (3, "+".to_string()), AinaToken::Toa => (3, "-".to_string()),
                AinaToken::Zidisha => (4, "*".to_string()), AinaToken::Gawanya => (4, "/".to_string()),
                _ => break,
            };
            if nguvu < min { break; }
            self.chukua();
            let kulia = self.usemi(nguvu + 1)?;
            if ["==", "!=", "<", "<=", ">", ">="].contains(&alama.as_str()) {
                kushoto = Usemi::Linganisha(Box::new(kushoto), alama, Box::new(kulia));
            } else if alama == "na" || alama == "au" {
                kushoto = Usemi::Mantiki(Box::new(kushoto), alama, Box::new(kulia));
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

#[derive(Clone, Debug)]
struct KaziHifen {
    vigeu: Vec<String>,
    mwili: Vec<Kauli>,
}
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
    let r = match s {
        "nyeupe" => "#ffffff", "nyeusi" => "#000000", "bluu" => "#38bdf8",
        "kijani" => "#10b981", "nyekundu" => "#ef4444", "njano" => "#facc15",
        "zambarau" => "#a78bfa", "machungwa" => "#fb923c", "kijivu" => "#94a3b8",
        "waridi" => "#f472b6", _ => s,
    };
    let sahihi = r.starts_with('#') &&
        ((r.len() == 7 || r.len() == 4) && r[1..].chars().all(|x| x.is_ascii_hexdigit()));
    if !sahihi { return Err("Tumia jina la rangi kama 'bluu', 'nyeusi' au msimbo kama '#112233'.".into()); }
    if r.len() == 4 {
        let chars: Vec<char> = r[1..].chars().collect();
        return Ok(format!("#{}{}{}{}{}{}", chars[0], chars[0], chars[1], chars[1], chars[2], chars[2]));
    }
    Ok(r.to_string())
}
#[derive(Clone, Debug)]
enum Kipengele {
    Kichwa(String),
    KichwaDogo(String),
    Aya(String),
    Kitufe(String),
    Sehemu { jina: String, watoto: Vec<Kipengele> },
}

#[derive(Clone, Debug)]
struct Tovuti {
    kichwa_kivinjari: String,
    kichwa: String,
    maelezo: String,
    kitufe: String,
    nyuma: String,
    rangi_maandishi: String,
    rangi_msingi: String,
    fonti: String,
    padding: u32,
    pengo: u32,
    upana: u32,
    pembe: u32,
    mpangilio: String,
    panga: String,
    ukubwa_kichwa: u32,
    kivuli: bool,
    maudhui: Vec<Kipengele>,
    bandari: u16,
}
impl Tovuti {
    fn mpya() -> Self {
        Self {
            kichwa_kivinjari: "Hifen".into(),
            kichwa: "Karibu Hifen".into(),
            maelezo: "Tovuti yako ya kwanza kwa Kiswahili.".into(),
            kitufe: String::new(),
            nyuma: "#0f172a".into(),
            rangi_maandishi: "#f8fafc".into(),
            rangi_msingi: "#10b981".into(),
            fonti: "system-ui, Arial, sans-serif".into(),
            padding: 32,
            pengo: 18,
            upana: 900,
            pembe: 20,
            mpangilio: "wima".into(),
            panga: "katikati".into(),
            ukubwa_kichwa: 48,
            kivuli: true,
            maudhui: Vec::new(),
            bandari: 8080,
        }
    }

    fn thibitisha_mitindo(&self) -> Result<(), String> {
        if !["wima", "mlalo"].contains(&self.mpangilio.as_str()) {
            return Err("mpangilio lazima uwe 'wima' au 'mlalo'.".into());
        }
        if !["katikati", "anza", "mwisho", "nyoosha"].contains(&self.panga.as_str()) {
            return Err("panga lazima iwe 'katikati', 'anza', 'mwisho' au 'nyoosha'.".into());
        }
        if self.padding > 128 { return Err("padding haiwezi kuzidi 128.".into()); }
        if self.pengo > 96 { return Err("pengo haliwezi kuzidi 96.".into()); }
        if !(320..=1600).contains(&self.upana) { return Err("upana lazima uwe kati ya 320 na 1600.".into()); }
        if self.pembe > 64 { return Err("pembe haiwezi kuzidi 64.".into()); }
        if !(20..=96).contains(&self.ukubwa_kichwa) { return Err("ukubwa_wa_kichwa lazima uwe kati ya 20 na 96.".into()); }
        if self.fonti.is_empty() || !self.fonti.chars().all(|c| c.is_ascii_alphanumeric() || " ,.-_'\"".contains(c)) {
            return Err("Fonti ina alama zisizoruhusiwa. Tumia mfano 'Arial, sans-serif'.".into());
        }
        Ok(())
    }

    fn rangi_kwa_css(s: &str) -> Result<String, String> { rangi(s) }

    fn html_kipengele(k: &Kipengele) -> String {
        match k {
            Kipengele::Kichwa(s) => format!("<h1>{}</h1>", html_salama(s)),
            Kipengele::KichwaDogo(s) => format!("<h2>{}</h2>", html_salama(s)),
            Kipengele::Aya(s) => format!("<p>{}</p>", html_salama(s)),
            Kipengele::Kitufe(s) => format!("<button type=\"button\">{}</button>", html_salama(s)),
            Kipengele::Sehemu { jina, watoto } => {
                let ndani: String = watoto.iter().map(Self::html_kipengele).collect::<Vec<_>>().join("");
                format!("<section class=\"sehemu\"><h2>{}</h2>{}</section>", html_salama(jina), ndani)
            }
        }
    }

    fn html(&self) -> String {
        let kichwa_kivinjari = html_salama(&self.kichwa_kivinjari);
        let (nyuma, maandishi, msingi) = (
            Self::rangi_kwa_css(&self.nyuma).unwrap_or_else(|_| "#0f172a".into()),
            Self::rangi_kwa_css(&self.rangi_maandishi).unwrap_or_else(|_| "#f8fafc".into()),
            Self::rangi_kwa_css(&self.rangi_msingi).unwrap_or_else(|_| "#10b981".into()),
        );
        let fonti = self.fonti.replace('"', "").replace('\'', "");
        let mwelekeo = if self.mpangilio == "mlalo" { "row" } else { "column" };
        let panga = match self.panga.as_str() {
            "anza" => "flex-start", "mwisho" => "flex-end", "nyoosha" => "stretch", _ => "center",
        };
        let kivuli = if self.kivuli { "0 20px 50px rgba(0,0,0,.24)" } else { "none" };
        let content = if self.maudhui.is_empty() {
            let kichwa = Self::html_kipengele(&Kipengele::Kichwa(self.kichwa.clone()));
            let maelezo = Self::html_kipengele(&Kipengele::Aya(self.maelezo.clone()));
            let kitufe = if self.kitufe.is_empty() { String::new() }
                else { Self::html_kipengele(&Kipengele::Kitufe(self.kitufe.clone())) };
            format!("{}{}{}", kichwa, maelezo, kitufe)
        } else {
            self.maudhui.iter().map(Self::html_kipengele).collect::<Vec<_>>().join("")
        };

        format!(r##"<!doctype html>
<html lang="sw">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{kichwa_kivinjari}</title>
<style>
:root {{ --nyuma:{nyuma}; --maandishi:{maandishi}; --msingi:{msingi}; --fonti:{fonti}; --padding:{padding}px; --pengo:{pengo}px; --upana:{upana}px; --pembe:{pembe}px; --ukubwa-kichwa:{ukubwa_kichwa}px; --kivuli:{kivuli}; --mwelekeo:{mwelekeo}; --panga:{panga}; }}
* {{ box-sizing:border-box; }}
body {{ margin:0; min-height:100vh; padding:clamp(16px,4vw,48px); display:flex; justify-content:center; align-items:center; background:var(--nyuma); color:var(--maandishi); font-family:var(--fonti); }}
main {{ display:flex; flex-direction:var(--mwelekeo); align-items:var(--panga); gap:var(--pengo); width:min(100%,var(--upana)); padding:var(--padding); border:1px solid rgba(255,255,255,.15); border-radius:var(--pembe); background:rgba(255,255,255,.045); box-shadow:var(--kivuli); text-align:{text_align}; }}
h1 {{ font-size:clamp(2rem,5vw,var(--ukubwa-kichwa)); line-height:1.12; margin:0; }}
h2 {{ font-size:1.35rem; margin:0; }}
p {{ max-width:68ch; margin:0; line-height:1.8; opacity:.9; }}
.sehemu {{ display:flex; flex-direction:var(--mwelekeo); align-items:var(--panga); gap:var(--pengo); width:100%; padding:var(--padding); border:1px solid rgba(255,255,255,.12); border-radius:calc(var(--pembe) * .7); background:rgba(255,255,255,.035); }}
button {{ background:var(--msingi); color:#fff; border:0; border-radius:9px; padding:14px 24px; font:inherit; font-weight:700; cursor:pointer; }}
button:focus-visible {{ outline:3px solid #38bdf8; outline-offset:4px; }}
@media(max-width:640px) {{ main,.sehemu {{ flex-direction:column; }} }}
</style>
</head>
<body><main>{content}</main></body>
</html>"##,
            padding=self.padding, pengo=self.pengo, upana=self.upana, pembe=self.pembe,
            ukubwa_kichwa=self.ukubwa_kichwa, kivuli=kivuli, mwelekeo=mwelekeo,
            panga=panga, text_align=if self.panga == "anza" { "left" } else if self.panga == "mwisho" { "right" } else { "center" }, content=content)
    }

    fn washa(&self, port: u16) -> Result<(), String> {
        if port == 0 { return Err("Namba ya seva haiwezi kuwa sifuri.".into()); }
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

struct Mtendaji {
    vigeu: HashMap<String, Thamani>,
    kazi: HashMap<String, KaziHifen>,
    kina_cha_wito: usize,
    tovuti: Tovuti,
}
impl Mtendaji {
    fn mpya() -> Self {
        Self {
            vigeu: HashMap::new(),
            kazi: HashMap::new(),
            kina_cha_wito: 0,
            tovuti: Tovuti::mpya(),
        }
    }

    // Kazi zina wigo wake: zinaweza kusoma vigeu vya nje, lakini vigeu vya ndani
    // na mabadiliko yake havivuji nje ya kazi. Kazi za toleo hili hazirudishi thamani.
    fn endesha_kazi(&mut self, jina: &str, hoja: Vec<Thamani>) -> Result<(), String> {
        if self.kina_cha_wito >= 64 {
            return Err("Kazi zimeitana kwa kina kinachozidi kikomo cha 64.".into());
        }
        let kazi = self.kazi.get(jina).cloned()
            .ok_or_else(|| format!("Kazi '{}' haijatengenezwa.", jina))?;
        if hoja.len() != kazi.vigeu.len() {
            return Err(format!("Kazi '{}' inahitaji hoja {}; umeleta {}.", jina, kazi.vigeu.len(), hoja.len()));
        }

        let vigeu_vya_nje = self.vigeu.clone();
        for (jina_kigeu, thamani) in kazi.vigeu.iter().zip(hoja) {
            self.vigeu.insert(jina_kigeu.clone(), thamani);
        }
        self.kina_cha_wito += 1;
        let matokeo = self.endesha(&kazi.mwili);
        self.kina_cha_wito -= 1;
        self.vigeu = vigeu_vya_nje;
        matokeo
    }

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
            Usemi::MantikiKinyume(x) => match self.tathmini(x)? {
                Thamani::Mantiki(v) => Ok(Thamani::Mantiki(!v)),
                _ => Err("Amri 'si' inahitaji thamani ya mantiki: kweli au sikweli.".into()),
            },
            Usemi::Mantiki(a, op, b) => {
                let x = self.tathmini(a)?;
                let xv = match x { Thamani::Mantiki(v) => v,
                    _ => return Err(format!("Upande wa kushoto wa '{}' lazima uwe kweli au sikweli.", op)) };
                // Tumia short-circuit: upande wa pili hausomwi ikiwa matokeo tayari yanajulikana.
                if op == "na" && !xv { return Ok(Thamani::Mantiki(false)); }
                if op == "au" && xv { return Ok(Thamani::Mantiki(true)); }
                match self.tathmini(b)? {
                    Thamani::Mantiki(yv) => Ok(Thamani::Mantiki(if op == "na" { xv && yv } else { xv || yv })),
                    _ => Err(format!("Upande wa kulia wa '{}' lazima uwe kweli au sikweli.", op)),
                }
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
                            ("sentensi", Thamani::Sentensi(_)) |
                            ("mantiki", Thamani::Mantiki(_)));
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
                Kauli::TengenezaKazi { jina, vigeu, mwili } => {
                    if self.kazi.contains_key(jina) {
                        return Err(format!("Kazi '{}' tayari imetengenezwa.", jina));
                    }
                    if ["AkiliMnemba.tengeneza_safu", "AkiliMnemba.jumlisha_safu",
                        "UkurasaMtandao.weka_kichwa", "UkurasaMtandao.weka_maelezo",
                        "UkurasaMtandao.weka_kitufe", "UkurasaMtandao.weka_rangi",
                        "UkurasaMtandao.washa_server"].contains(&jina.as_str()) {
                        return Err(format!("Jina '{}' limetengwa kwa amri ya Hifen.", jina));
                    }
                    self.kazi.insert(jina.clone(), KaziHifen { vigeu: vigeu.clone(), mwili: mwili.clone() });
                }
                Kauli::Tovuti(tovuti) => {
                    self.tovuti = tovuti.clone();
                    let port = self.tovuti.bandari;
                    self.tovuti.washa(port)?;
                }
                Kauli::Tumia(Usemi::Wito(j, args)) => {
                    let a: Vec<Thamani> = args.iter().map(|x| self.tathmini(x)).collect::<Result<_,_>>()?;
                    if self.kazi.contains_key(j) {
                        self.endesha_kazi(j, a)?;
                        continue;
                    }
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
