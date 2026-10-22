# Hifen 0.5 — Lugha ya programu ya Kiswahili

Hifen ni interpreter ya majaribio inayosoma faili za `.hifen`, iliyoandikwa kwa Rust na kutumia maktaba za kawaida za Rust pekee. Toleo hili lina aina ya mantiki na muundo mpya wa tovuti unaowekwa ndani ya faili moja.

## Kuendesha

Fungua PowerShell kwenye folder lenye `Cargo.toml`:

```powershell
cargo run -- endesha mifano/mwanzo.hifen
cargo run -- endesha mifano/kazi.hifen
cargo run -- endesha mifano/mantiki.hifen
cargo run -- endesha mifano/mabadiliko.hifen
cargo run -- endesha mifano/masharti_na_mizunguko.hifen
cargo run -- endesha mifano/tovuti.hifen
```

Tovuti inapowashwa, fungua `http://127.0.0.1:8080` kwenye kivinjari. Acha terminal wazi; bonyeza `Ctrl+C` ili kuzima seva.

## Aina za data

```text
tengeneza sentensi jina = "Asha"
tengeneza namba umri = 20
tengeneza desimali uzito = 42.5
tengeneza mantiki amesajiliwa = kweli
tengeneza mantiki amelipa = sikweli

andika(amesajiliwa)
andika(amelipa)
```

Thamani za mantiki ni `kweli` na `sikweli`. Pia unaweza kutumia `si` (NOT), `na` (AND), na `au` (OR):

```text
kama (amesajiliwa na si amelipa) {
    andika("Bado hujalipa.")
} vinginevyo {
    andika("Taarifa zako ziko tayari.")
}
```

Unaweza kulinganisha kwa `==`, `!=`, `<`, `<=`, `>` na `>=`. Thamani za mantiki zinaweza kubadilishwa kwa kutumia `badilisha`, lakini aina lazima ibaki mantiki.

## Masharti, mizunguko na kubadilisha vigeu

```text
tengeneza namba hesabu = 0
rudia (5) {
    badilisha hesabu = hesabu + 1
    andika(hesabu)
}
```

Idadi ya marudio ina kikomo cha mara 1,000,000. Vigeu vinavyobadilishwa ndani ya `kama` au `rudia` hubaki na thamani mpya baada ya kizuizi.

## Kutengeneza tovuti kwa faili moja

Huhitaji tena kurudia amri za `UkurasaMtandao...` kwa kila kipengele. Tumia muundo huu:

```text
tovuti "Tovuti Yangu" {
    ukurasa {
        kichwa "Karibu"
        maelezo "Hii ni tovuti yangu ya kwanza."
        sehemu "Kuhusu" {
            kichwa_dogo "Kuhusu mradi"
            aya "Tovuti imeandikwa kwa Hifen."
            kitufe "Anza"
        }
    }

    mitindo {
        fonti "Arial, sans-serif"
        rangi_ya_nyuma "#0f172a"
        rangi_ya_maandishi "nyeupe"
        rangi_ya_msingi "kijani"
        padding 24
        pengo 16
        upana 900
        pembe 20
        ukubwa_wa_kichwa 44
        mpangilio "wima"
        panga "katikati"
        kivuli kweli
    }

    seva 8080
}
```

Hifen huzalisha `<!doctype html>`, HTML na CSS kwa ndani. Msemaji hutumia sehemu hizi:

- `ukurasa`: maudhui ya tovuti.
- `kichwa`, `kichwa_dogo`, `maelezo`/`aya`, `kitufe`: vipengele vya ukurasa.
- `sehemu "Jina" { ... }`: kisanduku/flex container kinachoweza kuwa na vipengele vingine ndani yake.
- `mitindo`: `fonti`, `rangi_ya_nyuma`, `rangi_ya_maandishi`, `rangi_ya_msingi`, `padding`, `pengo`, `upana`, `pembe`, `ukubwa_wa_kichwa`, `mpangilio`, `panga`, na `kivuli`.
- `mpangilio`: `"wima"` au `"mlalo"`. Kwenye simu, muundo hupangwa wima ili usibane maudhui.
- `panga`: `"katikati"`, `"anza"`, `"mwisho"` au `"nyoosha"`.
- `kivuli`: `kweli` au `sikweli`.
- `seva 8080`: huanzisha seva ya ndani kwenye bandari hiyo. Unaweza kubadilisha namba ya bandari.

Rangi zinazotambulika ni `nyeupe`, `nyeusi`, `bluu`, `kijani`, `nyekundu`, `njano`, `zambarau`, `machungwa`, `kijivu`, `waridi`, au msimbo wa rangi kama `"#112233"`.

Kitufe kinachoonekana kwa sasa ni cha muonekano tu; matukio ya kubofya na kurasa nyingi bado havijatekelezwa. Seva inasikiliza `127.0.0.1` pekee na ni ya majaribio ya ndani, si matumizi ya production.


## Kazi unazounda mwenyewe

Hifen 0.5 inaruhusu kutengeneza kazi na kuitumia tena. Kazi inaweza kupokea vigeu vya kuingiza, na inaweza kuitwa mara nyingi:

```text
tengeneza kazi salimia(jina) {
    andika("Habari " + jina)
}

salimia("Asha")
salimia("Juma")
```

Kazi bila vigeu pia inaruhusiwa:

```text
tengeneza kazi salamu_ya_mwanzo() {
    andika("Karibu Hifen!")
}

salamu_ya_mwanzo()
```

Kanuni za toleo hili:
- Kazi lazima itengenezwe kabla haijaitwa.
- Idadi ya hoja unazotuma lazima ilingane na idadi ya vigeu vilivyo kwenye mabano.
- Kazi inaweza kusoma vigeu vya nje, lakini vigeu vya ndani ni vya kazi hiyo na havibadilishi vigeu vya nje.
- Kazi hazirudishi thamani bado; hutumika kutekeleza amri. Kurejesha thamani ni hatua inayofuata.
- Kina cha kuitana kwa kazi kimewekewa kikomo cha 64 ili kupunguza hatari ya mizunguko ya kujirudia isiyoisha.

## AkiliMnemba

```text
tengeneza mfumo_wa_ai = AkiliMnemba.tengeneza_safu(2, 3)
andika(mfumo_wa_ai)
```

Hii ni safu ya namba ya msingi kwa majaribio; bado si neural network wala mfumo wa kufundisha modeli.

## Ulinganifu wa nyuma

Amri za zamani za `UkurasaMtandao.weka_kichwa(...)` na `UkurasaMtandao.washa_server(...)` zinaachwa kwenye interpreter ili mifano ya awali iendelee kufanya kazi. Kwa miradi mipya, tumia `tovuti { ukurasa { ... } mitindo { ... } }`.

## Mipaka ya Hifen 0.4

Hifen bado ni interpreter ya hatua za awali. Bado hatujaongeza kazi za mtumiaji, modules/imports, forms zinazotuma data, database, au mafunzo halisi ya AI. Lengo la hatua hii ni kuimarisha aina za data na kutengeneza tovuti kwa muundo mmoja, unaosomeka kwa Kiswahili.
