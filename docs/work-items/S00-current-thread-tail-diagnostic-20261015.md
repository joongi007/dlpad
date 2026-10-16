# S00: 현재 소스 동적 thread admission·종료 tail 관측 준비

- 상태: prepared / native 미실행
- 분류: diagnostic
- 우선순위: P0
- 요구사항: R08, R13; S00 성능 예산
- 담당/수정 범위: p0_audit, 이 scratch 디렉터리만. Root는 공유 MSVC target critical section·통합·최종 검증 담당.
- 선행 작업: 현재 source 동결→default whole4 실제 성공·Root independent review→모든 current native/installer/package 작업 terminal. 예상 whole30 count/SHA를 기입하지 않는다.

## 사용자 결과

실패한 strict CPU 예산의 원인 분리를 돕는 관측 준비다. 제품 동작/성능 개선 완료가 아니다. 최초 snapshot 뒤 발견한 thread의 CPU/종료 tail을 추가 기록하며 작업별 CPU·quiet host·전수 커버리지는 여전히 미확인이다.

## 계약과 범위

현재 제품 Rust/source/schema/Cargo/lock/기본 feature 변경0. 기존 p0-frame-tail-diagnostic-v1 observer를 byte exact 복사한다. current-main-thread-cost-cause-v2 collector/gate는 현재 실제 whole 인자와 shared target feature proof를 받도록 최소 수정한다. 새 tail_contract는 기존 숫자 프로토콜의 한도·합계·부호있는 residual/시간/종료근거를 검사한다. feature-build는 기존 scripts/windows_msvc_frame_phase_build.py 파생으로 Root 단일 실행시 owned target-msvc default3를 exact backup→feature4→immutable feature CPU copy→finally exact default3 복구한다. 원 whole/GNU/project lock/source/helper pins는 복구 후 동일해야 한다. backup가 달라졌으면 기존 파일 복구로 덮어쓰지 않고 FAIL한다.

## 인수 조건

Root만 Cargo/CSC/native/벤치를 실행한다. default3 exact백업 before feature, build 실패시에도 복구 finally, 복구 실패는 FAIL 원자료로 보존한다. feature4는 fmt·strict clippy·locked workspace test·release이고 default-OFF first-paint-fixture만 기존대로 켠다. 실제 feature source before/after와 현재 whole 일치, current shipping3 exact 복구 후에만 단일60초/6window/private10k/readonlyDB/2PNG를 실행한다. 원1% single-core CPU/80MiB 예산·실제elapsed·150/170/45초 한도 불변. 원 strict CPU 실패를 진단 성공으로 지우지 않는다. lifetime128/poll750/window6, native HANDLE+creation identity·Wait0 종료·late admission CPU 제외·간격중 생성종료 누락·nonatomic signed residual·내용/secret 로그금지 유지. 중요한 관측손실/cap 초과는 성공으로 채우지 않는다.

## 검증과 handoff

수정 파일과 실제 정적 결과는 HANDOFF/static-result/candidate-pins에 기록한다. Cargo/feature/CSC/native/GUI/벤치 미실행. Root source/default3/all-current-native/package 완료 후 argv.json의 actual 인자를 채워 순차 실행한다. 구 whole25나 예상 whole30 수치/PE를 current 증거로 쓰지 않는다.

Root가 현재whole30/Rootreview 및 실제package/native terminal을 확인했다. source30을 유지하며 단일 feature critical section을 실행한다. 실제 남은디스크와 exact3백업/복구를 확인하고 결과를 갱신한다.

실제 feature v1 fmt/clippy0 뒤 test에서 engine staging budget fixture의512MiB set_len이 Windows StorageFull112로 실패했다(574 PASS/1 FAIL/17 ignored인 engine harness). release·collector 미실행이다. finally default3exactrestoretrue와 Root바이트재대조를 확인했다. `windows-msvc-ui-feedback-v30-frame-tail-feature-v1-20261015/` 원실패를보존한다. Root가 소유한24시간이전 engine debug incremental12개만 제거해 logical3,286,693,893B를회수했다(원PE3보존). sparse logical fixture 개선과 별도새feature실행이후속이다.

## 2026-10-16 source31 실행

source31 실제 whole/root review/source675/기본3PE를 확인한 뒤 모든 Cargo/native/package handle terminal에서 새 feature-build를 실행한다. command는 current30-frame-tail-prep-v1/feature-build.py --default-proof windows-msvc-ui-feedback-v31-whole-20261016/result.json --configuration sibling effective-configuration.json --output windows-msvc-ui-feedback-v31-frame-tail-feature-v1-20261016 --no-concurrent-dlpad-probes다. fmt/clippy/test/release 완료 전 계측GUI를 실행하지 않고 원3PE exact restore를 재대조한다. source31 sparse fixture가 전체기본시험에서 통과했다는 사실을 feature4 완료로 확대하지 않는다. 현재 실행중이며 결과 미확정이다. source32 후속 UI후보는 scratch/S에만 두고 이 critical section 동안 Cargo/native를 병렬실행하지 않는다.

source31 feature4 실제 완료: fmt/clippy/test/release 모두0,43harness/1,670PASS/0FAIL/42ignored. Root가 현재source675·원4로그·default3 바이트복구를 독립확인했다. [feature Root 검토](../validation/evidence/windows-msvc-ui-feedback-v31-frame-tail-feature-v1-20261016/root-review.json). 이어 단일60초 collector는 terminal0/DIAGNOSTIC_ONLY이며 실제6창 CPU/메모리 모두 예산충족으로 기록됐다. 평균Job CPU0.779958%·관측peak working set81,731,584B/private53,370,880B다. quiet_host=false/외부호스트부하미측정/default_budget_acceptance=false는 유지한다. retainedtail합 잔차0을 coverage완료·작업귀속확정으로 해석하지 않는다. 실제 원수치·DB/PNG·6창 별 Root검토는 후속이다. 이 양성 계측실행으로 이전 shipping strict실패를 삭제하거나 source32기본 예산해결을 선언하지 않는다.
