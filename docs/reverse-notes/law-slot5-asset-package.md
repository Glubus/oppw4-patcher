# Law Slot 5 Asset Package

## Etat actuel

Le pack fourni contient bien 1 modele et 8 textures/materials associees. Les fichiers ont ete copies hors dossier jeu pour staging non destructif :

- `target/law-slot5-assets/CharacterEditor/`
- `target/law-slot5-assets/MaterialEditor/`

Rien n'a ete installe dans `mods/CharacterEditor` ou `mods/MaterialEditor` pendant cette verification.

## Assets detectes

| Archive | Source | Cible privee | Hash source | Taille | SHA256 |
| --- | --- | --- | ---: | ---: | --- |
| CharacterEditor | `MPLC026_Law.g1m` | `MDLC999_Law_Custom.g1m` | `0x8df2d8cb` | 1,943,768 | `E6B05FA10CE2D99248B4F6DACC4A91F2F9CE2CCCAB10AD1B9C7B07FFC0E46246` |
| MaterialEditor | `MPR_Bound_Character_MPLC026Law_cloth_kidsalb.g1t` | `MPR_Bound_Character_MDLC999LawCustom_cloth_kidsalb.g1t` | `0x966d6276` | 699,104 | `9E66551B0E383794D790AA999D8DFEF4924DD45498AE87BCEE0BC7115CF2D062` |
| MaterialEditor | `MPR_Bound_Character_MPLC026Law_cloth_kidsnmh.g1t` | `MPR_Bound_Character_MDLC999LawCustom_cloth_kidsnmh.g1t` | `0x81b6a8a8` | 349,584 | `53964758BA557B7113CE0802FB5AE431329C7DCE5710D3187AB84D9AAAB2723A` |
| MaterialEditor | `MPR_Bound_Character_MPLC026Law_cloth_kidsocc.g1t` | `MPR_Bound_Character_MDLC999LawCustom_cloth_kidsocc.g1t` | `0x38a6472e` | 349,584 | `A256D23B17F0104FEE16100B50F203F4E8D60F5464114C9B99B1C2EA05362E11` |
| MaterialEditor | `MPR_Bound_Character_MPLC026Law_cloth_kidsrfr.g1t` | `MPR_Bound_Character_MDLC999LawCustom_cloth_kidsrfr.g1t` | `0xd341d61d` | 349,584 | `D54BB8E53B21399BF8D330B793CB50754E65EB301B40131C1196D8542CD27B87` |
| MaterialEditor | `MPR_Bound_Character_MPLC026Law_skin_kidsalb.g1t` | `MPR_Bound_Character_MDLC999LawCustom_skin_kidsalb.g1t` | `0x8b5cca29` | 16,777,272 | `5DFD68B6E1BE7DE6817ED43129D5562D6CAB0BF257C6E785CE1FAD542CB49123` |
| MaterialEditor | `MPR_Bound_Character_MPLC026Law_skin_kidsnmh.g1t` | `MPR_Bound_Character_MDLC999LawCustom_skin_kidsnmh.g1t` | `0x2798f5b7` | 22,369,592 | `A8FB9436C513B46A400D2FF9B4E44F70842918C235607715B3D0A61B6446337F` |
| MaterialEditor | `MPR_Bound_Character_MPLC026Law_skin_kidsocc.g1t` | `MPR_Bound_Character_MDLC999LawCustom_skin_kidsocc.g1t` | `0x90986e71` | 22,369,592 | `DEA9C81C7CDA2D92DD11334D2B61C77C27C527346C17573BCC162C2D7EC45B37` |
| MaterialEditor | `MPR_Bound_Character_MPLC026Law_skin_kidss4m.g1t` | `MPR_Bound_Character_MDLC999LawCustom_skin_kidss4m.g1t` | `0xbb2a9834` | 22,369,592 | `3C75CFDC9BF03F00595392F0D11F32AC3B57EA5F153EE7298E5CCB427DC07C29` |

## Verification RDB

`CharacterEditor.rdb` :

```text
files: 1
hash_matches: 1
hash_missing: 0
match hash=0x8df2d8cb file=MPLC026_Law.g1m
```

`MaterialEditor.rdb` :

```text
files: 8
hash_matches: 8
hash_missing: 0
match hash=0x966d6276 file=MPR_Bound_Character_MPLC026Law_cloth_kidsalb.g1t
match hash=0x81b6a8a8 file=MPR_Bound_Character_MPLC026Law_cloth_kidsnmh.g1t
match hash=0x38a6472e file=MPR_Bound_Character_MPLC026Law_cloth_kidsocc.g1t
match hash=0xd341d61d file=MPR_Bound_Character_MPLC026Law_cloth_kidsrfr.g1t
match hash=0x8b5cca29 file=MPR_Bound_Character_MPLC026Law_skin_kidsalb.g1t
match hash=0x2798f5b7 file=MPR_Bound_Character_MPLC026Law_skin_kidsnmh.g1t
match hash=0x90986e71 file=MPR_Bound_Character_MPLC026Law_skin_kidsocc.g1t
match hash=0xbb2a9834 file=MPR_Bound_Character_MPLC026Law_skin_kidss4m.g1t
```

## Conclusion

Ces 9 noms/hashes sont des assets officiels Law existants. Les remplacer directement dans les archives ou via un overlay global peut affecter Law normal, pas seulement le slot 5. C'est la raison des effets de bord observes plus tot.

Le preflight LinkData a aussi montre que l'ancien diagnostic `292` n'est pas
une cible propre dans le LinkData original :

```text
target_model=292
target_model_name=MPLC000_Luffy
target_model_status=occupied_by_other
recommended_model_target=730
```

Donc `292` doit etre considere comme un ancien id de probe runtime, pas comme
la cible finale du patch LinkData propre. La cible privee courante du package
offline devient `730`, sous reserve de validation au moment de la generation du
BIN.

La suite propre doit donc passer par une liaison de costume complete :

1. ajouter/brancher le variant costume `699` dans les tables LinkData utiles ;
2. associer `699` a un modele prive et a ses 8 textures/materials ;
3. eviter de remplacer globalement les hashes officiels partages ;
4. si on veut un vrai slot prive, creer une route private/alias pour les assets plutot qu'ecraser `MPLC026_Law.g1m` et les `MPR_Bound_Character_MPLC026Law_*`.

La prochaine piste concrete est de comparer les lignes LinkData d'Oni `586` / modele `308` avec la future ligne `699`, table par table, puis ne cloner que les champs assets necessaires.
