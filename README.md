# Hifen 0.3 — Lugha ya programu ya Kiswahili

Hifen ni lugha ndogo inayosoma faili za `.hifen` kupitia interpreter iliyoandikwa kwa Rust. Inatumia Rust standard library pekee; haina utegemezi wa crate za nje.

## Kuendesha kwenye Windows / VS Code

Fungua PowerShell ndani ya folder hili, lenye `Cargo.toml`, kisha endesha:

```powershell
cargo run -- endesha mifano/mwanzo.hifen
```

Jaribu vipengele vipya vya kubadilisha vigeu:

```powershell
cargo run -- endesha mifano/mabadiliko.hifen
```

Mifano mingine:

```powershell
cargo run -- endesha mifano/masharti_na_mizunguko.hifen
cargo run -- endesha mifano/tovuti.hifen
```

Tovuti inapowashwa, fungua `http://127.0.0.1:8080` kwenye kivinjari. Acha terminal wazi wakati seva inatumika. Simamisha seva kwa `Ctrl+C`.

## Vigeu na mahesabu

```text
tengeneza sentensi jina = "Asha"
tengeneza namba umri = 20
tengeneza desimali uzito = 42.5

badilisha umri = umri + 1
andika(umri)
andika(10 + 5 * 2)
```

`tengeneza` hutengeneza kigeu mara ya kwanza. `badilisha` hubadilisha kigeu kilichopo. Kwa sasa aina ya thamani lazima ibaki ileile: `namba` hubaki `namba`, `desimali` hubaki `desimali`, na `sentensi` hubaki `sentensi`. Hii husaidia kugundua makosa ya aina mapema.

## Masharti

```text
kama (umri >= 18) {
    andika("Unaruhusiwa.")
} vinginevyo {
    andika("Bado hujaruhusiwa.")
}
```

Alama za kulinganisha: `==`, `!=`, `<`, `<=`, `>`, `>=`. Sharti lazima liwe na matokeo ya kulinganisha.

Mabadiliko ya vigeu vilivyokuwepo yanabaki baada ya `kama` au `rudia`. Vigeu vipya vilivyotengenezwa ndani ya kizuizi huwa vya ndani ya kizuizi na havionekani nje yake. Kwa hiyo tengeneza kigeu cha kuhesabu kabla ya kuanza `rudia`.

## Mizunguko

```text
tengeneza namba hesabu = 0
rudia (5) {
    badilisha hesabu = hesabu + 1
    andika(hesabu)
}
```

Idadi ya marudio lazima iwe namba nzima isiyo hasi, na ina kikomo cha mara 1,000,000.

## Safu za AkiliMnemba

```text
tengeneza mfumo_wa_ai = AkiliMnemba.tengeneza_safu(2, 3)
andika(mfumo_wa_ai)
```

Hii hutengeneza safu ya majaribio iliyojaa sifuri. Bado si neural network, mfumo wa mafunzo ya modeli, wala GPU runtime.

## Tovuti

```text
UkurasaMtandao.weka_kichwa("Karibu Hifen")
UkurasaMtandao.weka_maelezo("Tengeneza tovuti kwa lugha ya Kiswahili!")
UkurasaMtandao.weka_kitufe("Anza kujifunza")
UkurasaMtandao.weka_rangi("nyeusi")
UkurasaMtandao.washa_server(8080)
```

Seva ya sasa husikiliza `127.0.0.1` pekee na ni ya majaribio ya ndani, si matumizi ya production. Kitufe kinachoonekana kwenye tovuti bado hakijaunganishwa na kitendo cha kivinjari.

## Kujenga executable

```powershell
cargo build --release
.\target\release\hifen.exe endesha mifano/mwanzo.hifen
```

## Mipaka ya toleo la 0.3

- Hii ni interpreter ya hatua za mwanzo, si compiler ya machine code.
- Bado hakuna kazi za mtumiaji, modules/imports, arrays za jumla, database, au mfumo kamili wa AI.
- Sarufi na taarifa za makosa zitaendelea kuboreshwa kadri tunavyoongeza vipengele.
