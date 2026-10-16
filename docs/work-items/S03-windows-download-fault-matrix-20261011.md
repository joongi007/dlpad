# S03: Windows HTTP 단절·만료 및 writer 실패 복구

- 상태: completed
- 분류: verification
- 우선순위: P0
- 요구사항: R04, R08, R10, R11
- 담당/수정 범위: root 인터페이스/통합/실행/검증; diagnostic_inputs는 /tmp HTTP시험 이식후보(local_media_failures.rs), release_preparation은 /tmp 새Windows writerfixture/test후보. 프로젝트원파일은root가순차통합한다.
- 선행 작업: 실제Windowswhole1011/0/24와 fault증거표. 현재완료bundle2044cbab는이후회귀추가전검증snapshot이며기존.git보존.

## 사용자 결과

네트워크 만료·중간 단절과저장writer오류 후에도실패를완료로표시하지않고원본/partial을보존하며같은job의재시도가새attempt로정상복구되는지 실제Windows엔진에서검증한다.

## 계약과 범위

Rust네이티브제품/loopback-only server·4MiB합성미디어·64KiBwriterfixture·이번작업의새tmp만사용한다. 외부영상/사용자Documents/실제disk채움없이시험한다. writer112는주입이며physicalfilesystemfull로표현하지않는다. Cargo workspace/lock/schema는root만조정하고현재변경계획없음. HTTP agent와writer agent는서로다른/tmp파일만수정하며새native실행은root가순차수행한다. Git정상화/공개push/CI/updater실행은범위밖이다.

## 인수 조건

Windows실행testcount0을PASS로처리하지않는다. 실제HTTP403·단절·Range재개·samejob/DBreopen 및writerpartial→Failed→새attempt2→Downloaded를관측한다. cancel/deadline/소유프로세스Job0·상한·원본/foreign보존을확인한다. 기존SQLitequota오류/게시복구증거를물리diskfull로소급하지않는다. root전체fmt/Clippy/test/release와자료SHA/원로그를대조하며제품byte변경시새후보검증한다.

## 검증과 handoff

현재독립증거감사: Windowswhole의SQLitequota/reopen2·nativepartialpublication/identity는실제PASS. Linux전용disk_failure와Unix전용local_media_failures는Windows에서0tests였으므로새범위가필요하다. 준비된설치ZIP과Gitbundle을완료산출물로보존하면서이후검증을추가한다. 후보미통합/미실행단계이며완료로보고하지않는다.


HTTPport를통합했다. 첫Windows실행은accepted socket의nonblocking모드차이로요청이읽히지않아FAIL을보존했다. root가accepted stream blocking+500msread/write timeout을명시한후 `windows-msvc-http-fault-blocking-accepted-20261011` 실제1testPASS(18.72초):403·32KiB단절·Failed/artifacts0·DBreopen·samejobattempt2/positiveRange/최종byte검증. root원raw/actualcount1/ToolSHA/source/Job자연0를대조했다. realFFmpeg+yt-dlp loopback만사용했으며8hwallsoak/physicaldiskfull주장이아니다. 별도writerfixture/test를root통합중이며reparse검사는symlink_metadata로수정했다.


`windows-msvc-writer-fault-first-20261011` 실제1testPASS(0.32초). root는keptownedfixture의DB Downloaded/attempts2/artifact1·integrityok·64KiBpartialSHA·보호sentinel/복사toolSHA·원source추출2회·자연Job0를대조했다. Win32112는명시softwareinjection이고물리diskfull은미증명이다. 새example/test는자동Cargo발견을사용하며Cargo.toml/lock/schema는변경하지않았다. 현재새whole검증중이며기존CPU/installerbyte와비교후현재사용자후보근거를확정한다.


`windows-msvc-download-fault-matrix-whole-20261011` fmt/Clippy/test/release모두PASS. root원raw/618source/원GNU/lock/3artifactSHA검토:42harness1011passed/0failed/26ignored. 새무시HTTP/writer2시험은위별도실제실행PASS다. CPU·CLI·installer3release파일모두기존검증/실제설치후보와byte동일하여제품GUI/설치/idle시험을불필요하게반복하지않는다. 본작업의bounded실패복구범위완료이며8hwallsoak/physicaldiskfull/OSsleep을완료로주장하지않는다.
