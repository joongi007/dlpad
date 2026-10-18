# S11: Windows 앱 업데이트 전환과 복구

- 상태: in-progress, 사용자 공개 승인 후 CI·배포 인수 중
- 분류: feature
- 우선순위: P1
- 요구사항: R08, R10, R11
- 담당/수정 범위: 주 담당; app_update/app_install 공유 계약, app entry/수명 연결, Cargo/workflow/CONTEXT/ADR/문서, 전체 통합 검증
- 선행 작업: 공개 v0.1.0 및 GitHub Pages, signed metadata/transport/staging/새 bundle 준비 인수

## 사용자 결과

서명 검증→다운로드→새 버전 준비→작업 idle 확인→UI 종료/기존 버전 보존→새 UI readiness 확인→실패 복구를 구현한다.

## 계약과 범위

사용자가 2026-10-18 업데이트 구현을 명시 요청했다. 현재 v0.1.0은 첫 updater 지원 버전의 1회 수동 설치가 필요하다. 첫 지원 릴리스는 업데이트 및 관련 UX에 집중하고 별도 대형 기능은 섞지 않는다. 주 담당이 공유 schema·Cargo/lock·인터페이스를 조정한다. 기존 .git/index는 보존하고 정상 worktree를 사용한다. 공개 태그/Release·운영키 외부 공급은 구현·합성 검증과 구분하며 기존 비밀 전송 승인 범위를 유지한다. 검증은 임시 폴더/합성 fixture/private desktop만 사용한다.

## 인수 조건

앱의 idle/작업 중 gate, 서명·크기·해시 오류, 중단·stale 상태, 준비/실행 실패, 기존 A와 사용자 미디어·도구·프로필 보존을 검증한다. 원 owner 종료·새 버전 readiness·복구 결과를 구분한다. UI/OS 동작은 단위 검사로 대체하지 않는다. 운영키 없음을 임의 test key 신뢰로 대체하지 않는다.

## 검증과 handoff

진행 중. D: 여유 약1.6GiB, /tmp 여유610GiB를 확인했다. 기본 PATH의 rustc는 없으며 기존 pinned Rust/Windows 도구 경로를 확인 후 사용한다. Git 정상 worktree/main은 fc6f559이며 root의 과거 index·불명 lock을 삭제하지 않는다. 낮은 D: 공간에 새 Cargo cache를 만들지 않는다.


2026-10-18 중간 기록: 원본상태 보존 후 독립 worktree3개에서 bootstrap/journal/presentation을 병렬 구현했다. Root가 이를 복사·검토하고 CPU app entry와 실제 frame readiness, 중앙 문구·고정 운영 공개키·0.1.1 workspace/lockfile을 조정했다. 새 crate/버전 의존성 추가0. 운영 비공개키는 로컬 Windows DPAPI 암호화 파일로만 보관하고 외부 전송0; 로컬 서명 도구와 실패 경로 회귀를 추가했다. 공개 릴리스/운영 자동 서명 공급 완료와 구분한다.

실제 Windows native-engine-v2 strict engine Clippy0, 업데이트89PASS/4FAIL을 관측했다. 원 실패 로그는 ignored `.local-tools/update-activation-20261018/native-engine-v2/`에 보존했다. journal publication reader의 공유 예약, directory ReOpenFile Winerror5, Windows에서 사용할 수 없는 fixture `?` 경로가 각각 원인이며 수정·재검증 중이다. profile URI의 canonical verbatim prefix도 개선했다. native-v3 전체 strict Clippy에서는 host의 collapsible_if4개로101을 관측했으며 타입 오류는 없었으나 후속 테스트는 실행되지 않았다. 성공으로 바꾸지 않는다. Linux app_update68PASS는 이 OS 전환 검증을 대체하지 않는다.


최신 Root 인수(native-v6): `cargo clippy --workspace --all-targets --features dlpad/cpu --locked --offline -- -D warnings`, `cargo test --workspace --features dlpad/cpu --locked --offline`, CPU/installer dev build 및 engine lib build가 모두0. 원 stdout test-result 합계1,831PASS/0FAIL/44ignored(별도 filtered child 포함)다. Root가 직접 원 로그와 자연 Job0/forced=False/input desktop/source 불변을 확인했다. 이후 code13/gen0 wire 결함을 독립 감사로 발견해 정확한 알림만 예외 허용하는 회귀를 추가했고, 전역 복귀 배너·프로필 실제 컬럼 preflight를 보완해 재검증한다. v6의 통과를 이 후속 코드까지 검증한 것으로 쓰지 않는다.

실제 최신 Windows standalone(C: native-v6 lib) 7case: handshake/진행률/준비/RestartAccepted, 같은 부모3회 순차 UI, 초기 실패, 시작 timeout, 다른 profile ID, 부모 종료의 nested Job, child crash37/owned tail 종료에서 원 exit37+Job0을 확인했다. 원 로그는 `.local-tools/update-lifecycle-fixture-20261018/native-fixture-v4/fixture.log`다. 최신 named 공유 회귀12개도 v6에서 통과했다. signed B의 commit/coldstart/rollback은 별도 fullhost 실제 시험 중이다.

로컬 키 공급: Windows actual DPAPI 합성4회귀 native-key-v2 PASS(변조·entropy 불일치·기존 파일 보존 포함), private Job 자연0/입력 desktop 불변. 최초 v1은 embedded Python의 모듈 검색 경로 오류로 실행되지 않았고 v2 도우미에서 경로를 명시한 뒤 통과했다. Linux signer17PASS·local signing guard3PASS·release policy5PASS, source-only docs1,136 PASS를 실제 실행했다. 운영 비공개키는 검사의 대상이 아니다.

저장 공간: D: 여유1.2GiB로 내려가 새로 만든 C: 사용자 Temp 작업전용 폴더에 기존 캐시를 복사했다. `.local-tools/update-activation-20261018/c-target.json`에 독점 생성 nonce/경로를 기록했고 기존 캐시·사용자 데이터 변경0이다. 프로젝트 TEMP/TMP를 네이티브 도우미에 명시해 사용자 Documents로 리디렉트된 시스템 기본 temp에 의존하지 않는다.


최신 전환 proof: `.local-tools/update-host-fixture-20261018/run-1792313770810094027/host-fixture.log`의 전체7 CASE_PASS를 Root가 직접 읽었다. 성공 A→서명 B→RuntimeReady→선택 확정→cold B, 시작 실패/실행 중 crash의 복귀와 failed sequence 재시도 거절, hash mismatch, Cancel ACK, busy 중복, stale generation을 확인했다. SQLite와 media/tool sentinel 보존, 동일 공급 요청/worker수, 원 source 불변, private Job 자연0/forced0/입력 desktop 불변이다. 이 fixture는 고정 시험키와 합성 PE/프로필을 사용하며 운영 릴리스 다운로드나 실제 UI rollback 시험이 아니다. 진단용 임시 stderr는 제거했고 이후 최종 전체 검증을 수행한다.

최신 private GUI debug 검수: `.local-tools/update-ui-review-20261018/gui-v6/` 실제26 PNG·원부모/자식4process 자연exit0·두Job0·입력 desktop 불변과 profile.theme=Dark를 확인했다. Root도 Light800 및 Dark200% PNG를 직접 열어 글자/겹침을 확인했다. 오래된 릴리스에 설치를 권하던 링크/안내를 신규일때만 설치, 기존/더낮은버전은 릴리스 보기로 교정했다. 작은 창에서 변경내역/본문 nested-scroll과 literal Markdown은 후속 UX 항목이다. 실제 DPI/물리 IME/접근성 reader/Release 성능 및 signed UI 활성화 화면은 이 검수로 완료 처리하지 않는다.

추가 원인/수정: (1) 원 FILE_READ_ATTRIBUTES HANDLE로 내용을 읽어 commit이 OS5로 실패 → 원 HANDLE의 readonly ReOpenFile 및 full128 ID/경로/길이/hash 대조로 수정, 실제 Windows 공유/rename 실패 회귀 추가. (2) 복귀 Failed13/generation0가 wire decoder에 거절 → 정확13/0 예외와 회귀. (3) stage Interrupted가 Disk5로 표시 → Cancelled1 매핑과 회귀. 최초 실패 원 로그를 보존했다.

기본OFF `app-update-fixture` 별도 bin과 독립 CI build/run capture를 추가했다. 기존 fixed7 제품 source/패키징 그래프를 늘리지 않으며 합성 키와 시험 exe는 제품ZIP에 포함하지 않는다. Root Python capture 회귀11PASS, source-only docs1,136PASS. 새 CI wrapper의 실제 Windows 실행과 최종release build/전체 테스트는 진행 중이다.


최종 로컬 인수 `native-final-v1`: fmt check, 전체 workspace/all-targets strict Clippy(CPU), fixture strict Clippy, 전체 workspace CPU test, 전체 workspace CPU release build 5명령 모두0. 원 stdout43 harness 합계1,837PASS/0FAIL/44ignored(필터된 내부 자식 포함), source_unchanged=true·private Job 자연0/forced0/입력 desktop 불변. 경과257.2초. 실제 ReleaseCPU11,082,240B SHA25665c9bd4d54c5f26b503817bea6ddceaa6a2804b29949c3cd97711e6afcff695f. 공개v0.1.0 CPU10,430,464B 대비651,776B 증가(약0.62MiB); 새crate/version추가0이며 이는 전체 업데이트 변경 비용이다. 부모/client 각각100ms poll·idle network0·변경 없는 UI repaint0; 부모 추가 메모리/CPU는 별도 실제 관측한다.


CI 자체 실행 인수 `native-ci-fixture-v1`: 기본OFF fixture strict Clippy/build 두 원Job 및7case별 원Job 모두 exit0/자연0/readerdrained/forced0. Root가 `.local-tools/update-activation-20261018/ci-fixture-v1/whole.json`의 passed=true와 각 raw stdout·lifecycle를 확인했다. outer private host도 source불변/자연Job0/입력desktop불변, 경과98.4초. embedded Python 격리 검색 경로 실패를 막는 helper 경로 명시와 `python -I` 회귀도 포함했다.

최종 Release private GUI `gui-release-v1`: Root가 exact CPU SHA65c9bd4d…cff695f의 실제 Light preview PNG와 두 run의 원JSON을 읽었다. 원부모/자식4process 자연exit0/두Job0·강제정리0·입력desktop불변, Light/Dark/200%/작은창·Dark 재시작복원·오래된release 보기 링크를 확인했다. 별도 독립 review와 SQLite읽기 검증도 보존했다. 운영signed 다운로드/전환을 GUI에서 클릭한 증거와 물리 IME/DPI 인수는 아니다.

quiet10.0077초 참고 관측: 부모 CPUdelta0s(계측 해상도 내), working set12,046,336B/private commit2,039,808B; UI CPUdelta0.046875s(단일core0.4684%), working set83,365,888B/private commit52,924,416B. 이 working set 합계는 공유 page를 중복 계산하며 privateRSS가 아니다. 앱 기본 metrics는 켜져 있었고 관측기 UIA/캡처/입력은 구간 내0. updater만의 순수 추가비용, 전체80MiB 목표 인수 또는 일반 성능보장으로 쓰지 않는다.

추가 Root Python 정책 최종7suite: 총82개 실행/81PASS/1개 Windows전용 skip, 모든7 subprocess exit0. 원로그 `python-final-policy.log`. 별도 실제Windows DPAPI4회귀 및 CI fixture 원Job 회귀로 해당Windows경계를 확인했다.

원본 로컬 증거 포함 `python3 scripts/check_docs.py` 1,231 documents PASS/원upstream notices133 hash일치. installer launch 읽기 감사에서는 Started=원프로세스resume 통지이며 직접GUI PID/readiness ACK가 아니고 원Job 전체수명감독임을 확인했다. production argv는argv0만이므로새부모→UI계층과충돌하는특수인자없음. 실제최종설치형→앱 실행은새패키지 인수에서추가확인한다.


공개 보류: 자동 승인 검토가 새0.1.1의 main push+공개태그 생성을 거절했다. 이유는 기존 구현/Git 연동 승인만으로 이번 정확한 공개 릴리스 배포의 별도 승인 확인이 되지 않는다는 것이다. 거절 명령은 CreateProcess 이전에 중단되었고 push/tag/commit을 실행하지 않았다. 외부 push/태그/릴리스는 재시도·우회하지 않는다. 로컬 전용 커밋과 검수 실행파일·검증 자료 정리는 계속하며, 실제 새 hosted CI/설치ZIP/공개 metadata는 이 공개 승인 이후 단계다. 비공개 운영키의 외부 공급은 별도이며 이번 승인 요청에 포함하지 않는다.


최종 Release4exe의 AMD64/GUI2·console3 구분과 OS DLL allowlist 검증을 통과했다(`final-runtime.json`). 최초 검증도우미가 console helper를 GUI2로 검사해 실패했고 정확한console3계약으로재검증했다. 제품PE오류가 아니다. 검수용 exact CPU copy는 `.local-tools/update-activation-20261018/review-v0.1.1/dlpad.exe`이며 SHA는검증Release와동일하다. 기존앱을종료한뒤실행하며 CI 설치ZIP/공개릴리스라고표현하지않는다. 새 capture 출력은 `.gitignore`의 `/ci-update-captures/`로명시해CI source상태검사와분리했다.

2026-10-18 후속 사용자 응답으로 main·v0.1.1 태그·설치파일 공개를 명시 승인받았다. 기존 거절은 승인 전 이력이다. 9fe2045 소스와 새태그를 normal atomic push했고 원격 main·태그를 대조했다. 비공개 운영키 외부 공급은 포함하지 않는다. 새 CI·패키지 검수·로컬 서명 후 공개 metadata와 설치ZIP 공개를 진행한다.
