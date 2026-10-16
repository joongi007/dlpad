# S00: Windows 입력 유효성 속성 연결

- 상태: completed
- 분류: accessibility
- 우선순위: P0
- 요구사항: R07, R08
- 담당/수정 범위: root — Cargo.toml, Cargo.lock, vendor/accesskit_windows, 해당 공급·검증 문서; diagnostic_inputs — /tmp 후보
- 선행 작업: 높이 입력 실제 Windows 오류/정상 조회 실패 증거 확보

## 사용자 결과

입력 오류를 고쳤을 때 Windows 접근성 클라이언트도 정상 유효성을 읽는다.

## 계약과 범위

locked accesskit_windows0.29.2의 기존 FullDescription 연결은 유지하고 IsDataValidForForm30103을 AccessKit Invalid와 연결한다. None은 valid, True/Grammar/Spelling은 invalid다. getter뿐 아니라 값 변경 이벤트 연결도 검증한다. root가 공유 Cargo/lock·공급 원본·라이선스·비용을 조정한다. 후보는 아직 채택·제품 변경 완료가 아니다.

## 인수 조건

실제 Windows height0→1080에서 original provider COM30103 false→true,30159 오류→제거와 원 PID/HWND/opaqueRuntimeID·focus·값 보존을 확인한다. actual property dispatch 및 invalid 세 종류 회귀도 필요하다. 새 의존성 버전이나 기존 registry cache를 수정하지 않는다. vendored 공급 해시·원문 license·용량을 기록한다.

## 검증과 handoff

native v1은 수집기의 잘못된 예상 문구로 실패했고 원 결과를 보존했다. v2 Light57255/Dark23693은 실제 core 오류 문구로 교정한 뒤30103을0과1080에서 모두false로 읽어 실패했다. 30159는0에서 정확한 오류,1080에서 빈 문자열이다. 두 app는 실패 정리에서 소유 Job으로 종료되어 자연 종료 성공으로 계산하지 않는다. 내부 actual UI 회귀 통과를 이 OS 실패 대신 쓰지 않는다.

locked node.rs properties!에는 IsDataValidForForm 매핑이 없고 unmatched property는 empty Variant를 반환한다. /tmp/dlpad-windows-validation-provider-20261012 후보는 getter·properties macro 연결과 실제 consumer/dispatch regression을 제공한다. 공급20파일232,025바이트와 BSD 추가 고지·MIT/Apache 원문 확보 필요성을 기록했다. 아직 root 통합·Cargo/native 검증은 수행하지 않았다.

- root가 registry archive checksum 및 원20파일 해시를 대조하고 vendor/accesskit_windows를 통합했다. .cargo-ok는 archive에 없는7바이트 설치 마커라 제외했다. upstream 자체 Cargo.lock은 독립 테스트를 위해 원문 보존했다. getter/properties mapping·실제 consumer dispatch 회귀만 node.rs에 추가했다. 기존 registry cache는 수정하지 않았다.
- 세 원문 MIT/Apache/Chromium BSD를 정확 commit5801c4a49ff9b2c9ec3f77b3c31272d0da4c5cd3 기준으로 보존했다. Chromium 원문1,559바이트 SHA845022e0c1db1abb41a6ba4cd3c4b674ec290f3359d9d3c78ae558d4c0ed9308와 notice manifest crate binding도 연결했다. 배포 전체 legal completeness를 주장하지 않는다.
- root workspace patch/exclude와 root Cargo.lock 해당 package source/checksum 두 줄만 변경했다. 버전·dependency 목록은 byte-level 대조로 동일하다. 추가 runtime getter/property compare 한 개이고 새 thread/I/O/의존성 버전은 없다. 최종 binary 크기는 새 release 후 기록한다.
- targeted42843은 feature 미활성으로 package 조회 실패,88310은 non-workspace dev-dependency 시험 제한으로 실패,31638은 upstream 자체 lockfile의 cesu8 cache 부재로 offline 실패했다. 제품 regression 성공으로 계산하지 않는다. 별도 upstream manifest의 --locked 시험을 유지하고 프로젝트 전용 cache에 필요한 테스트 의존성 다운로드를 허용한 새 실행을 시작했다. root lock 변경 범위는 동일하다.
- 다운로드 후7720은 실제 컴파일에서 consumer Node에 invalid()가 직접 없는 E0599로 실패했다. root가 원 consumer API의 data().invalid()로 수정하고 새 offline/locked 시험을 실행했다. 잘못된 후보 API를 단위 검사 성공으로 기록하지 않는다. 원 공급 inventory와 patch 이전 source를 보존한다.
- targeted32837 exit0: 독립 vendor manifest의 --locked/--offline Windows 시험1개 통과. 실제 consumer tree의 None/True/Grammar/Spelling과 native property dispatch nonempty를 검사했다. 원 Job 자연 종료·source 불변 확인. 실제 COM client 및 변경 이벤트 전달은 아직 별도 검증 대상이다.
- root Cargo.lock SHA는5ae2c3af01813478787aba6498aac183cbf540e5976e996d15952f2fe6241ed2이며 원d3ae0580b678632e14e7f52c498fb04f64aba1e6d0fd1d7a3e74b8c65c4c58e1에서 해당 package 두 줄만 제거했다. byte-level 대조로 다른 version/dependency는 동일하다. 공급 근거와 standalone test의 dev-dependency 범위를 vendor DLPAD-PATCH.md에 기록했다.

- 전체80951 및 독립78291 exit0: fmt/clippy/test/release,43 harness/1161성공/0실패/30ignored와 source/lock/raw/artifact 대조 통과. 독립 vendor unit1개는 별도32837 증거다. CPU9,710,080바이트 SHA `6855305065ace8613e1e68b71e0f294e9072911695090ead89d1a2d2fd4818b0`; 이전 input-focus-error 대비512바이트 증가.
- 새 app provider-fixed Light3198/Dark18594를 실행했다. 오류0→정상1080의 COM30103/30159·원값/focus/identity·first/restart·자연 종료를 엄격 재검증하며 아직 성공으로 기록하지 않는다. 값 변경 이벤트 listener 후보는 별도 /tmp/dlpad-height-validity-change-event-20261012에 준비했고 actual event delivery는 미검증이다.

- provider-fixed Light3198/Dark18594 collector exit0. 실제 first/restart30103은0=false→1080=true,30159는정확오류→빈문자열로 복구했다. 원값/focus/opaqueID·재시작/테마·자연 종료/Job0 유지. 독립v2는 relative core pin을 absolute-only 함수에 넘겨 실패했다. 원후보/실패보존 후 v3는 knowncore 경로만 절대화하고 초기960 summary와 실제 phase800 geometry/PNG를 구별해 양쪽 PASS_RAW_PROVIDER_DB_ONLY_PIXEL_PENDING으로 검토했다. 각 evidence에 root-independent-native-review.json 보존.
- root가 Light/Dark invalid/corrected PNG4장을 직접 봤다. red marker 제거와 값 복구는 확인했으나 오류 설명이 clip 밖이라 별도 height-error-context 작업을 연다. physical input/full UI 인수는 아니다.
- event91632는 관리형30103 lookup 미지원으로 실패했다. native COM 대안76791은 callback runtime 변수의 C# definite assignment 오류로 compile 실패했다. 원결과를 보존하고 null 초기화만 한 새v3 collector14634를 실행했다. 실제 event delivery는 아직 미검증이며 기대값 또는readback을 event로 대체하지 않는다.
- native COM event14634 exit0 및 독립31926 exit0: first/restart에서 원 입력의 실제30103 false→true callback 두 개씩·정확 sender/PID/runtimeID·16개 상한·listener 제거·failure 없음과 기존0/1080 readback/COM/DB/테마/자연 종료를 검증했다. 원.NET lookup 실패와 v2compile 실패는 보존한다. old value는 API가 제공하지 않아 관측하지 않았으며 native route는 원 copied observer ABI로 연결한다. metadata readback을 callback 성공으로 대체하지 않았다. scoped provider 동작은 인수했지만 실제 reader 사용성과 전체 UI/릴리스는 별도다.

- 이 작업의 Windows 속성 연결·consumer dispatch·실제 값 복구·변경 이벤트와 공급/lock 인수는 완료했다. helper glyph clipping, 물리 reader/IME, 전체 UI/UX·패키지·릴리스 게이트는 해당 별도 작업에서 유지한다. 이후 height-context test-only source 변경은 이 고정한 app 증거를 새 전체 검사로 소급하지 않는다.
