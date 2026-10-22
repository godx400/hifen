# Hifen 0.6 — Lugha ya programu ya Kiswahili

Hifen ni interpreter ya majaribio inayosoma faili za `.hifen`. Imeandikwa kwa Rust na hutumia maktaba za kawaida za Rust pekee. Toleo 0.6 linaunga mkono aina za data za Kiswahili, masharti, mizunguko, kazi zinazoundwa na mtumiaji, na amri ya `rudisha` inayoweza kurudisha thamani kutoka ndani ya kazi.

## Kuendesha

Fungua PowerShell kwenye folda iliyo na `Cargo.toml`:

```powershell
cargo run -- endesha mifano/mwanzo.hifen
cargo run -- endesha mifano/mantiki.hifen
cargo run -- endesha mifano/masharti_na_mizunguko.hifen
cargo run -- endesha mifano/mabadiliko.hifen
cargo run -- endesha mifano/kazi.hifen
cargo run -- endesha mifano/kazi_ya_jibu.hifen
cargo run -- endesha mifano/tovuti.hifen
```

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

Thamani za mantiki ni `kweli` na `sikweli`. Pia unaweza kutumia `si` (kinyume), `na` (na), na `au` (au):

```text
kama (amesajiliwa na si amelipa) {
    andika("Bado hujalipa.")
} vinginevyo {
    andika("Taarifa zako ziko tayari.")
}
```

Unaweza kulinganisha kwa `==`, `!=`, `<`, `<=`, `>` na `>=`.

## Masharti, mizunguko na kubadilisha vigeu

```text
tengeneza namba hesabu = 0
rudia (5) {
    badilisha hesabu = hesabu + 1
    andika(hesabu)
}
```

Idadi ya marudio ina kikomo cha mara 1,000,000.

## Kazi unazotengeneza mwenyewe

Kazi huruhusu kutumia msimbo tena. Kwenye toleo 0.6, kazi zinaweza kurudisha thamani kwa kutumia `rudisha`:

```text
tengeneza kazi jumlisha(a, b) {
    rudisha a + b
}

tengeneza namba jibu = jumlisha(5, 3)
andika(jibu)
```

Matokeo ni `8`. `rudisha` inaweza pia kurudisha sentensi, desimali, au thamani ya mantiki:

```text
tengeneza kazi ni_mtu_mzima(umri) {
    kama (umri >= 18) {
        rudisha kweli
    }
    rudisha sikweli
}

andika(ni_mtu_mzima(21))
```

Kanuni za kazi:
- Kazi lazima itengenezwe kabla ya kuitwa.
- Idadi ya hoja lazima ilingane na vigeu vilivyo kwenye mabano.
- Kazi inaweza kuitwa kama amri au kutumika ndani ya hesabu/kauli nyingine.
- Ikiwa kazi inatumika kama thamani, lazima itekeleze `rudisha`; kutorudisha thamani husababisha kosa linaloeleweka.
- `rudisha` inaweza kutumika ndani ya masharti na mizunguko iliyo ndani ya kazi; inamaliza kazi mara moja.
- `rudisha` haiwezi kutumika nje ya kazi.
- Kazi huhifadhi wigo wake wa vigeu: mabadiliko ya ndani hayavujii nje.
- Kina cha kazi kujitana kimewekewa kikomo cha 64.

## Kutengeneza tovuti kwa faili moja

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

Hifen huzalisha `<!doctype html>`, HTML na CSS kwa ndani. Weka `fonti`, rangi, `padding`, `pengo`, `upana`, `pembe`, `ukubwa_wa_kichwa`, `mpangilio`, `panga`, na `kivuli` ndani ya sehemu ya `mitindo`. Tovuti ya majaribio hupatikana kwenye `http://127.0.0.1:8080`; acha terminal wazi na tumia `Ctrl+C` kusimamisha seva.

Rangi zinazotambulika ni `nyeupe`, `nyeusi`, `bluu`, `kijani`, `nyekundu`, `njano`, `zambarau`, `machungwa`, `kijivu`, `waridi`, au msimbo kama `"#112233"`.

## AkiliMnemba

```text
tengeneza mfumo_wa_ai = AkiliMnemba.tengeneza_safu(2, 3)
andika(mfumo_wa_ai)
```

Hii ni safu ya namba ya msingi kwa majaribio; bado si neural network wala mfumo wa kufundisha modeli.

## Mipaka ya Hifen 0.6

Hifen bado ni interpreter ya hatua za awali. Bado tunahitaji kuboresha ujumbe wa makosa, modules, ukaguzi wa aina za data kabla ya utekelezaji, fomu za tovuti zinazotuma data, database, na mafunzo halisi ya AI. Tovuti na seva ya sasa ni ya majaribio ya ndani, si matumizi ya production.
