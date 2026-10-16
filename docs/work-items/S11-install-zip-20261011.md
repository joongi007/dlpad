# S11: 원본 binding 기반 설치 ZIP

## 2026-10-12 최신 UI 묶음 준비

root가 실제 installed5 bytes/namespace/재검증 journal SHA와 원 raw Job/pins를 독립 검토해 `root-installed-bytes-review.json`에 보존했다. 이어 current native observation을 독립 reviewer로 재실행하여27개 records와5 payload IDs의 결합을 확인했다. historical ownership/atomic namespace/deletion authority는 false다. root가 initial/target-derived-journal/forged-journal-display/installed/verified 다섯 PNG를 직접 확인했다. 이는 반응형 경로 source1097의 scoped 인수다. 이후 확인한 corrupt DB 이력 진입 수정은 이 묶음에 없으므로 최종 사용자 링크로 승격하지 않는다. explicit sentinel receipt 부재와 미검증 OS/서명 조건도 유지한다.

fresh package 독립 검토는 PASS_BOUNDED_ARCHIVE_AND_PROOF_ONLY로 종료했다. 새 install ZIP은16,466,220B / SHA77fc837e1c7e5a9241ffb731e24e87a1a42c9b3e8aa2f5d43d974571810477b4다. archive/proof 인수만이며 실제 설치·UI 최종 인수는 아직 없다. 기존 링크를 최신 인수 파일로 바꾸지 않는다.

root가 반응형 경로/history/도구 배치 수정의 latest whole 1,097/0/28을 독립 검토했다. `/tmp/dlpad-responsive-ui-final-package/pipeline.py`로 동일 proof의 actual graph 세 명령과 fresh preview/payload/install ZIP 생성을 실행했다. 원 기존 설치 ZIP과 사용자 링크는 유지한다. 새 파일은 아직 독립 package 검토·실제 설치/재검증·현재 UI 인수를 기다리므로 정식 릴리스나 사용자 테스트 준비 완료로 표현하지 않는다. 의존성/schema/lock 변화는 없으며 selected graph258과 whole-lock missing43을 구분한다. 인수 조건은 현재 바이너리/source/graph/manifest binding, 실제 5파일 설치와 재검증, 원 데이터·native ID 보존 및 가상 화면 root 검토다.

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R13
- 담당/수정 범위: root 통합; release_preparation /tmp 후보. scripts/generate_install_payload.py, scripts/package_install_payload.py, scripts/test_package_install_payload.py
- 선행 작업: Windows 전체 검증과 원본 binary/notice binding, S07 packaged installer

## 사용자 결과

설치 ZIP 전체를 풀고 dlpad-install.exe를 실행하여 설치 위치 하나를 지정한다. journal 위치는 자동으로 정한다.

## 계약과 범위

기존 원본 binary SHA를 생성 binding에 포함하고 정확한 7개 ZIP member를 고정 metadata로 포장한다. Python 표준 라이브러리만 사용한다. STORED는 압축 비용과 버전 차이를 없애며 총 payload 크기 수준의 다운로드 비용을 갖는다. 기존 파일 덮어쓰기, symlink, 경로 탈출, 변조된 SHA를 거절한다. 서명·법적 전체 coverage·공개 배포 준비 완료를 주장하지 않는다.

## 인수 조건

원본 보존과 ZIP SHA/CRC/member 검증, 잘못된 입력 실패 및 기존 출력 보존. 새 실제 Windows binary payload의 설치·재검증과 private GUI 증거 확인.

## 검증과 handoff

하위 후보 Python 18건 통과 보고를 받았다. root 독립 Python 18건도 통과했다. Windows 전체 검사 41 harness에서 1009 passed, 0 failed, 19 ignored와 fmt/clippy/release가 통과했다. 실제 payload/native 검증은 진행 중이다. Git 원격과 서명·clean OS 검증은 별도 gate다.

2026-10-11 최신 current whole616/1009 기반 설치ZIP 생성: dlpad-install-local-av-v6-20261011.zip, 16,334,636B, SHA2562003946ebca70b146ba433edf664df5a24ecb5187cfdc9f7cc8306a14a8a48f5. 정확7members independent ZIP CRC/SHA 검증과 원 generated source 보존 PASS. manifest ab1b2816e4a4595a6c263cf2e86e7cf4293d9e87b8f1d23095c7e5aaf985fcc2, binding c34e45e36efd808e1d9a8a11e9a07dd63bf4592545133dd90a1652fd201f07a4. unsigned/release_ready=false/complete legal coverage=false. 실제 생성5entry 전체 설치·재검증은 다음 native 인수로 남는다.

최신 full-installed-local-av-v6 실제 private GUI install/reverify 자연0·Job0 통과. root가 installed5와 journal5 SHA를 원 명세/receipt에 독립 대조하고 initial/verified PNG를 직접 확인했다. unsigned/cleanOS/사용자 OS 최종 인수는 별도이며 전체P0 완료가 아니다.


### 진단 viewport 수정 후 새 설치 ZIP

`dlpad-install-diagnostic-final-20261011.zip`16,334,124B/SHAd240f82be570170a7c549545ff9cca71012445d5fc3b53daaba63aff28fd0a23는 새whole1011/0/24의 CPUe241510c…를 포함한다. preview17,380,308B/SHA01a4f5f325985f40aa11e073a7cf7aa5c04f13690d153a376ef1ac716ea156e3, actualgraph258 selected notices/missing0이며 wholelockmissing43/legalcompletefalse를 유지한다. root는 ZIP7member/CRC/각SHA/generated원본byte를 대조했다. manifest01e7ba8b59666d7b9d1710d4f3bc46582f72613fc5dbc33ffa6372d52ed29b1b, bindinge3b78969cd804d04b6dae02775627b8c323d37bf5881b8f755ca0dab8e871306, installerSHA는b47c85ce…로동일하다. 실제새5파일설치/재검증은후속한다. unsigneddevelopment후보이며정식release_readyfalse다.


`windows-msvc-diagnostic-final-installed-payload-20261011` 실제새5파일install/reverifyPASS_SCOPED. root는원raw/전체pin/원본source616·설치registered정확8namespace·5payloadSHA/bytes·5journalSHA·initial/verifiedPNG를직접대조했다. 새CPUe241510c…가설치되었으며기존source7파일보존·forged journal display의worker권한불변을확인했다. cleanOS/signing/업그레이드/최종사용자UI는미검증이다.

### 실행 계획 스크롤 수정 후보

[새 설치 ZIP](../validation/evidence/dlpad-install-plan-scroll-20261011.zip): 16,334,636 bytes, SHA256 `0f1e7e18d1e0ea0f228734ea7904ddf564948ce278feab880cac11fb15e4941a`. `python3 /tmp/dlpad-package-plan-scroll.py`는 현재 whole1013/0/26와 실제 graph258을 대조해 preview ZIP → 5파일 payload → 7member 설치 ZIP을 create-only 생성했다. root가 ZIP의 모든 member 원본 바이트/size/SHA와CRC를 독립 검증했다. CPU b8434e…9,537,024B, installer b47c85…4,310,016B. 현재 새 묶음의 실제 설치·재검증은 진행 전이며, 기존 개발설치 인수와 혼동하지 않는다. unsigned/release_ready=false, upgrade/updater 제공 아님.

새 묶음 실제 인수 완료: `windows-msvc-plan-scroll-installed-payload-20261011` root-review PASS_SCOPED. exact registered8 namespace, manifest5 entry size/SHA, journal5 SHA, 원본/pins/rawnatural0/Job0/privateclosed, initial/verifiedPNG를 root가 대조했다. 검증된 설치 ZIP이며 cleanOS/서명/upgrade/자동updater 인수와는 별개다.

## 진단 커서 교정 설치본

`dlpad-install-diagnostic-caret-20261011.zip`은 16,338,220B, SHA256 `e7bf7c3bade68841f165ff038d92d675a85f8837cf73d4e752652fc7279d79dc`이다. 루트가 ZIP 7개 원소의 정확한 byte/size/CRC와 whole의 CPU SHA 연결을 검증했다. `windows-msvc-diagnostic-caret-installed-payload-20261011/root-review.json`에서 실제 5개 파일 설치·재검증, 등록 namespace8개, journal5개 SHA, 생성 원본7개 보존, natural exit/Job0를 검토했다. 실제 installed/verified PNG도 확인했다. 화면 인수는 `windows-msvc-diagnostic-caret-current-gui-20261011/root-review.json`에 연결한다. 전체 검사1,015/0/28과43harness다. unsigned 개발 설치본이며 기존 묶음 upgrade·clean OS·서명·법적 완결·전체P0 완료를 주장하지 않는다.

## 최신 수동 도구 선택 설치 후보

[설치 ZIP](../validation/evidence/dlpad-install-manual-tool-preferences-20261011.zip) 16,423,212B SHA256 `ff4f7bb50b8387b583f021627e5cd2c4fd020adf0e3ce8173b8b1d07a1d79cbf`. whole-fifth 1,055/0/28(43 harness), fmt/clippy/release와 private GUI5단계·실제SQLite root 인수를 결합했다. graph258/engine86 source620 root 인수, exact7 ZIP member CRC/streamedSHA/size/stamp 별도 검토. 실제 새 Windows 설치5files·재검증·원generatedSHA·변조journal무시·자연exit0/Job0·private desktop 원본 보존과 installed/verified PNG root 검토 PASS_SCOPED. manifest `97bcbe605e4a57d4c032c6938b7fb31d388e3064e2495f96799d33f5a3af7cd7`, binding `6269373ac5a1f15a76bef70c1753c332089ee5ac91252de55bb3259f7af2acdf`. unsigned development_only/release_ready=false; upgrade/cleanOS/서명/전체P0 완료 아님. 압축해제전체→dlpad-install.exe→새 설치 폴더→bundle/registered/dlpad.exe.

## 도구 첫 진입 개선 후보 검증

최신 whole은 1,063 passed/0 failed/28 ignored(43 harness), CPU SHA806bf03e44b897cb082d92148563b044c1f7b47ce7cb607d36d1319b1081e129다. 새 설치 ZIP과 graph를 생성했으며 root 독립 검토와 실제 설치를 진행한다. 첫 설치 시험은 sandbox의 WSL interop socket failed1로 observer 컴파일 전에 실패했다. 실패로그와 출력은 보존했으며 별도 fresh 출력에서 허용된 private Windows 시험을 다시 실행한다. 실제 설치 인수 전 README의 현재 설치 링크는 이전 검증본으로 유지한다.

## 최신 도구 첫 진입 설치 후보

[설치 ZIP](../validation/evidence/dlpad-install-tools-first-open-20261011.zip) 16,423,212B SHA256 `d104726294d164c9054eefb08847cf3cb7cbdd8dfcb29137e42f4dfcb74817d7`. root가 exact7member streamedSHA/CRC/size/stamp1980/modes/PE 및 실제 graph258/86/258·notice258의 원 SHA와 whole1063/0/28·source620 결합을 독립 재실행했다. 실제 `windows-msvc-tools-first-open-installed-payload-second-20261011/root-review.json`에서 설치5files·재검증·registered namespace8·journal5SHA·원7generated SHA·원raw natural0/Job0·private desktop 보존과 installed/verified PNG를 확인했다. 첫 sandbox interop 실패 출력은 보존했다. CPU9,625,600B이며 직전 후보와 크기동일하다. manifest `4799881c9d04ac8b461e5432c5f7016f6b3878a9866ec9421e4b3f3d245d23ea`, binding `c2e8534739d5c5a0e600377e691db7a7aed79e07c70ce400cf8146bc57ca0d69`. unsigned development_only/release_ready=false, cleanOS/upgrade/서명/법적완결/최종사용자UI는 별도다.

## 실행 파일 선택 통합 설치 후보

[설치 ZIP](../validation/evidence/dlpad-install-tool-executable-picker-shell-hint-final-20261011.zip)16,451,372B SHA256 `780e0e1f1beb1c560e9d5bdc8876bc8a8021747df9171cce856b71f9b7f6c123`. whole-fifth1089/0/28/source623 CPU294184…9,653,760B·graph258/86/258·notice선택누락0/wholelock누락43을rootboundedarchive검토했다. 실제설치5/재검증/원33핀/등록namespace8/journal5SHA·자연0/Job0 및installed/verifiedPNG직접검토통과. 별도nativeHANDLE27의현재ID/등록payload5·recordID도독립검토했다. 실제GUIv10모달10·선택/취소/저장/재실행/owner-close5phase·DB5를검토해README현재링크로채택했다. manifest87b96c21e70e06dba2bd00889f111923fc06c44b45c07d031b21db50a2056384, binding1ee9cedfc5c3e8b875a54c3d051e055c39fc9462e888163a84d4e556c9342167. unsigned/release_ready=false 및cleanOS/upgrade/서명/법적완결은별도미완료다.

## 2026-10-12 최신 History 복구·반응형 UI 설치 검토

주 담당이 최신 whole(1,101 성공/0 실패/28 ignored)의 설치 후보를 별도로 검토했다. `review_install.py`는 실제 5파일 바이트·재검증 receipt 대조에 통과했고, `review.py --root …owned-8099ec95… --observation …/native-observation.json`는 현재 27레코드와 등록 payload 5개의 native ID 대조에 통과했다. initial/target-derived-journal/forged-journal-display/installed/verified PNG 5개를 직접 확인했다. 근거는 `windows-msvc-responsive-ui-history-fix-20261012-install/root-review.json`과 `windows-msvc-responsive-ui-history-fix-installed-native-identities-20261012/root-review.json`이다. 과거 소유권·삭제 권한·서명·제품 UI 최종 인수는 이 결과로 증명하지 않는다.

### 실제 viewport UI 후보 설치 준비

`/tmp/dlpad-real-viewport-final-package`와`/tmp/dlpad-real-viewport-final-installer-review`는 fresh real-viewport-final-20261012 출력 및 명시적 current whole proof만 사용한다. 기존5파일 설치/reverify 관측기와27개 native identity probe를 유지한다. 주 담당이 package pipeline의 bounded tag/proof/current source/정확 artifact SHA 및 argv 단계를 검토하고 pure guard7개를 재실행해 통과했다. installer-review 폴더에는 discover 대상 pure test가0개여서 별도 테스트 통과를 주장하지 않는다. 실제 포장·native 설치는 최종 whole/UI 인수 후 실행하며 현재 FAIL proof를 사용하지 않는다.

### 最新 UI 빌드 설치 의존성 확인 시작

주 담당이 최신 `windows-msvc-real-viewport-keyboard-ui-whole-v2-20261012/result.json`의 정확한4검사/current source/원본 logs를 확인한 뒤, 준비된 `real-viewport-final-20261012` pipeline의 graph 단계를 실제 실행했다. 설치 ZIP 생성/사용자 설치본 갱신/시각 인수 완료는 아직 아니다. 새 GUI 캡처 및 DESIGN 비교 검토와 병행하며 source 변경이 필요한 결함을 찾으면 그 변경 후 전체 검증으로 재귀속한다.

실제 graph/package/review 완료. bounded 원본 ZIP 검토 `PASS_BOUNDED_ARCHIVE_AND_PROOF_ONLY`: installation ZIP16,469,804 bytes/SHA256 `4b5a799a9d9d1fd712fa62dbc913b56a7b4b0ee2d438b67e6f002557e6e155f5`; preview ZIP17,515,988 bytes/SHA256 `56d483026a3ffa9a094c6e24d08770674f6ea6429e722e37eebe03f9716fd43a`. 설치7 members/preview466 members/selected graph258개 notice missing0 확인. whole-lock superset missing43은 별도이고 complete legal/public release false 유지. 원본 리뷰는 /tmp/dlpad-real-viewport-final-package/real-viewport-final-20261012-review.json. 이어서 소유 임시 폴더의 실제 native 설치 시험을 시작했으며 종료 전 설치 성공으로 기록하지 않는다.

실제 install collector 종료0/REVIEW_REQUIRED. 독립 review_install.py: `PASS_INSTALLED_FIVE_BYTES_AND_REVERIFY_RECEIPTS_ONLY`, installed5 files/original33 pins 확인. 주 담당이 initial/installed/verified/forged-journal-display/target-derived-journal PNG5개를 직접 봤고 root-install-pixel-review.json에 SHA와 제한을 기록했다. 설치 UI 정보 계층·간격은 후속 시각 개선 대상으로 남기며 UI 완성으로 주장하지 않는다. current native identities 독립조회 시작; historical sentinel identity는 이 자료로 증명하지 않는다.

Current native identities collector 종료0. 주 담당의 root-review.json은 원본 compile/native-identities Job guards/current source/모든 pins를 독립 확인한다. current files의 identity 일치만 인수하며 historical ownership/deletion authority/public readiness는 false 유지한다.

새 density source의 current whole 검토와 packaging pure8 tests를 주 담당이 확인한 뒤 download-density-final-20261012 graph 단계를 시작했다. 앞선 real-viewport-final ZIP은 density 개선 전 빌드이며 최신 UI 포함이라고 안내하지 않는다. 현재 새 GUI/source freeze 동안 Cargo/schema/worker 수정 없음.

최신 density ZIP 독립 review PASS_BOUNDED_ARCHIVE_AND_PROOF_ONLY: install16,470,316 bytes SHA256 b32a3001c6ad88117b8fc08b60d5918f188f736723d4621c186f8db849f3a0e1; preview17,516,500 bytes SHA256 e7ad9ddf9d0b92aac78a447f37ba9075305e72b3e44d2af6a263a52705f5125c. 실제 install collector를 시작했다. 종료 전 성공으로 기록하지 않으며 unsigned/public false를 유지한다.

Density install collector 종료0/REVIEW_REQUIRED; 독립 review_install.py 실행 중. GUI5PNG/direct review 및 current native identities는 아직 후속 확인이 필요하다.

Density 설치5PNG를 주 담당이 직접 보고 root-install-pixel-review.json에SHA/제한을 기록했다. current native identities root-review.json도 originalJob guards/pins와27records/5payload IDs 일치 검토PASS. historical ownership/atomic namespace/deletion authority는미증명이다. 설치UI의간격·긴경로표현추가개선과후속empty-row빌드의재포장은남아있다.
