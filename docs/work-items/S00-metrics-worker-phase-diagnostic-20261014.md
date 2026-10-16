# S00: 실제 metrics worker 단계별 진단 (후보)

- 상태: done / source v23 합성 fixture 진단 범위
- 분류: diagnostic
- 우선순위: P0
- 요구사항: R01, R08, R11
- 담당/수정 범위: windows_auto — 두 분리 후보 파일; Root — format/공유 S·통합·native collector·최종 검증/문서
- 선행 작업: Root 인터페이스 검토, 실제 MSVC strict 검증 및 owned collector 승인

## 사용자 결과

사용자 동작을 변경하지 않는다. API microprofile에서 확인하지 못한 live metrics caller-worker의 volume/discovery/route/sample/mailbox/repaint 단계 wall time과 같은 worker CPU를 제한된 synthetic fixture에서 구분한다. 원 default 18창 비용 FAIL 해결 또는 성능 원인을 확정하는 작업이 아니다.

## 계약과 범위

기존 first-paint-fixture feature만 사용하며 새 dependency/feature/Cargo 변경이 없다. 기존 observer baseline, volume 독립성, Unicode alias, one worker/latest-only mailbox, cadence, Stop/CloseWait를 유지한다. 중앙 UI 문자열·engine·schema·DB·interval/cache를 변경하지 않는다. Probe owner/clock은 !Send/!Sync이며 caller worker에서 생성·읽기·종료한다. root/data/media 모두 새 owned fixture이며 marker/create_new report만 허용한다.

## 인수 조건

fixture OFF shipping CPU는 모듈/호출을 cfg에서 제외한다. fixture ON도 명시 env 없이 admission I/O나 CPU clock allocation을 하지 않는다. disabled/missing marker/existing report/incorrect ownership는 refusal이며 기존 report를 바꾸지 않는다. 64 ticks 이후 동작은 정상 수행하되 기록만 누락 수로 집계한다. 최대 숫자값 64×6 report가 64KiB 이내여야 한다. CPU unavailable은 null이며 finite clock zero는 CPU 사용 없음의 증명이 아니다. 에러/취소/정지 시 report 실패가 기존 metrics 정책을 바꾸지 않아야 한다.

## 검증과 handoff

후보 4개 테스트 작성, 실행하지 않음. Windows 0.61.2 binding/기존 feature 정적 확인만 완료. Linux rustfmt 실행은 command not found(127); MSVC rustfmt/clippy/test는 Root 담당. live/source/evidence/native/Cargo를 변경·실행하지 않았다. 실행 명령/collector environment/계측 한계는 HANDOFF.md에 기재한다.

Root 실제 명령은 app lib가 아닌 `--bin dlpad-cpu`와 `--features cpu`/`cpu,first-paint-fixture`를 사용한다. frozen HANDOFF의 lib 예시는 실행하지 않는다. Git exit128을 보존한다.

Root가 frozen pins와 원 accessible_metrics.rs를 대조하고 정확한 Rust 1.94.0 rustfmt로 별도 Root 복사본을 포맷한 뒤 S 두 파일만 overlay했다. 실제 `run_isolated_v2.py metrics-only-green-v1 20261014`: 새 probe **4/0/0**, 기존 metrics **15/0/0**, 전체 workspace all-target strict clippy0. 원로그·소스·candidate map·native Job0/private input unchanged를 [독립 검토](../validation/evidence/managed-tool-isolated-metrics-only-green-v1-20261014/root-review.json)했다. 첫 Root helper setup은 pins.files를 map으로 가정해 실패했고 native0회인 별도 namespace에 보존했다. Live/현재 v22 설치본은 변경하지 않았다. first-paint-fixture compile/기존 worker 전체 회귀·실제 private worker report·전체 배포 후보 검증은 후속이며 원 strict18구간 성능 실패가 해결된 것으로 표현하지 않는다.

Root가 소스670을 채택한 뒤 일반whole1,536/0/42·43harness를 독립검토했다. 기존 scripts/windows_msvc_frame_phase_build.py의 소유권 marker/고정 원binary를 대조하고 별도cached target을 명시 reuse했다. 진단feature 전체4명령 실제 **1,570/0/42·43harness**, [독립 검토](../validation/evidence/windows-msvc-metrics-worker-phase-feature-20261014/root-review.json)와 원11입력/current shipping3 보존을 확인했다. 고정 진단CPU10,336,256B(SHA e4a9ee3ad9694bf455503cb3dfd115478744067891b3ac24776ece22856145a9)는 일반CPU10,242,560B보다93,696B 크며 패키지에 넣지 않는다. 현재 marker 명시·새ownedfixture·18초관측 후 자연종료/report첫읽기 collector를 실행하며 실제원인/기본비용PASS는 아직 판단하지 않는다.

실제 collector 명령(HANDOFF exact argv) compile0/idle0/자연앱0/Job0/private input unchanged, source670/current3·원49pins를 Root가 다시 대조했다. 소유 empty media fixture에서 실제17ticks report를 자연종료 뒤만 읽었고 [독립 검토](../validation/evidence/current-metrics-worker-phase-private-v1-20261014/root-review.json)했다. volume/discovery/route/sample/mailbox/repaint-dispatch를 포함한 outer wall합13,113,900ns/회평균771,405.88ns. finite CPU delta0은 실제비용0의 증거가 아니며 repaint 이후UI/layout/paint/present와10,000목록·전체프로세스/원default18 strictFAIL은 이측정의 인수대상이 아니다. 관측주기/cachepolicy 변경은 하지 않았다. 이후UI정렬수정은 별도작업이며 이측정은 v23 source에 고정해 보존한다.
