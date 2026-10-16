# S00: 현재 Windows scan CloseWait와 Stop ACK 재검증

- 상태: done (current worker/CloseWait scoped 검증; 전체 P0와 분리)
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — 인터페이스·게시·통합·원소유 Windows native 실행·최종검증; soak_recovery_audit — /tmp feature producer/기존observer adapter 초안. 예정 scripts/windows_msvc_closewait_feature_build.py 및 scripts/windows_msvc_scan_closewait_collect.py; 기존 Rust/Cargo/schema/observer를 수정하지 않는다.
- 선행 작업: 최신 기본 source의 Windows 전체4검사 완료 proof. source 동결 후 feature 빌드와 다른 native 측정은 직렬 조정한다.

## 사용자 결과

scan 진행 중 닫기/중단 후 종료는 기존 원 worker ACK와 종료 대기를 보존하며 사용자 데스크톱을 띄우지 않는다.

## 계약과 범위

기존 cpu,library-scan-closewait-fixture feature만 사용한다. completed current default proof/raw4·정확 source/artifact를 강제하고 별도 프로젝트소유 MSVC target namespace/owner marker와 immutable feature CPU를 만든다. 원default CPU/CLI·GNU/lock은 보존한다. 실제MSVC whole4/strictalltargetClippy를 실행하며 실패증거를 보존한다. collector는 이완료featureproof와기존 Windows original-capability scan CloseWait observer/seed/원smallfilearchive 계약을 재사용한다. fallback desktop/global input/강제정리를 성공으로 표현하지 않는다. 새dependency/features/Cargo/lock/domain schema 변경은 없다.

## 인수 조건

원 PID/creation/HWND/Job·scan 진입·UI 중단 후 종료·원workerStopACK·원DBrollback/파일보존·자연Job0/private/input복원을 실제검증한다. 정확 source/feature/default artifact 핀과 raw원명령을 root/독립대조하며 PNG/readonlyDB를 root검토한다. 단위Host due 시험만으로 실제OSCloseWait를 대체하지 않는다.

## 검증과 handoff

현재 /tmp 준비 단계다. source/API/seed/proof 계약을 먼저 검토하고 root가게시·실행한다. 아직 feature whole/build/GUI 완료 근거가 아니다.

root가원stage SHA/AST·producer/collector currentproof/raw4/strictargv/각op source 및원/복사/compiled핀post검사를읽어2Python을게시했다. 원default/firstpaint target은수정하지않고별도closewait typedowner와immutable CPU를사용한다. seed는원example을MSVC target/offline/locked로컴파일실행하며원CAP/root 경계를확인한다. compile/seed/gui 원rawguard·복사proof원decodedSHA·source전후/자연Jobprivate/inflight/원StopACK·smallownedarchive를인수한다. native실행은아직미수행이며현재기본18측정종료뒤순서대로진행한다.

실행전독립감사에서outer collector의scenario/release_sent/원manualreset event3개/timeout경과를추가확인할필요를찾아root가strictguard로보강했다. 현재별도target 최초whole빌드진행중이며fmt/clippy원exit0·test/release는진행중이다. 실제완료전인수로표현하지않는다.

별도CloseWait MSVC whole4 실제exit0·833/0/13·root/독립인수PASS다. featureCPU59305a82047f6a0d4ad092989efcaa4f4729c4c4ffb0a4e709f50ebb14114c88/8,729,600B는기본342…보다18,944B증가하며기존defaultOFF fixture기능이다. defaultCPU/CLI/GNU/lock보존확인. release case 실제privateGUI collector진행중이며case실행완료전nativeCloseWait 인수는false다.

currentsource의실제CloseWait release3656ms/timeout20009ms두case를root/독립인수했다. 각원ACK전20회alive·ACKfalse, actualfirstobservation, 원3manualresetevents, ACK후naturalexit0/Job0/privateinput불변, 원DBold1행/path/size/nativeidentity 논리보존 및newobserv미게시를확인했다. root는두PNG/readonlyarchiveDBintegrity·SHA불변/원8files와archive SHA를직접대조했고원fixture를삭제하지않았다. 현재값은기존featureCPU59305…에결합된scope이며새설치검사source의일반OS/physical/전체P0인수로확대하지않는다.
