# S10 CPU 보고서 화면 페이지화

- 상태: done
- 분류: bug
- 우선순위: P0
- 요구사항: R13
- 담당/수정 범위: stress_recovery; crates/app/src/accessible_diagnostics.rs, crates/app/src/accessible_preview.rs, scripts/cpu_report_stress_probe.py, 이 문서 및 전용 evidence. root는 catalog/통합 빌드 담당.
- 선행 작업: S10-cpu-report-stress full-10 실제 1.9MB 보고서 VmHWM511.8MiB

## 사용자 결과

최대 유효 진단 자료의 전체내용을 이전/다음/직접 페이지 선택으로 읽는다. 화면은 UTF-8 경계에 맞춘 최대8KiB만 layout하고 복사/저장은 전체 정제된 자료를 유지한다.

## 계약과 범위

ADR-0004 진단 보고서 계약, 원 report/preview String/전체 copy/export bytes 보존. 페이지 경계는 최대2MiB 입력 안에서 8192B씩 생성하고 UTF-8 boundary를 최대3B 후퇴하여 유한 index를 유지한다. 빈 문서는1빈페이지. 편집/omit/format/worker return 시 처음페이지로 재설정. 공유 schema/Cargo/entry/helper 수정 금지.

## 인수 조건

UTF-8 모든 bytes 재구성/8192경계/빈문서/상한초과/선택clamp/전체 export 불변/format·edit·omit·workerreturn reset 시험. 동일 실제 최대보고서 격리GUI에서 전체copy/export 보존과 메모리 비용 비교. unit으로 GUI 성능이나 다른OS 완료를 주장하지 않는다.

## 검증과 handoff

기능회귀 fixture full-10의512MiB급 HWM은 실제 실패 baseline이다. 가설: 전체preview galley 비용, format별galley캐시, report/JSON 복제. 화면페이지 layout 한변수 변경 후 root frozen 새binary로 동일GUI 비용을 비교한다. 구현 직후에는 Rust/GUI 미검증으로 기록했고 아래 실제 검증 결과로 갱신했다.

## 완료 근거 (Linux 제한)

root 요청으로 PreviewPages를 accessible_preview.rs에 분리하여 실행계획 UI에서도 재사용한다. shared catalog/entry/plan 통합은 root가 담당했다. 새 코드는 root CPUclippy/71tests/release 검증 뒤 SHA166946…으로 고정했다. 실제 paged-3/4 GUI exit0/PASS: 225페이지의 직접 첫/끝 및 이전/다음, readonlyfirst8192B/last6845B 전체JSON 일치, 전체copy/export 정확보존, collision/원job 불변/정상종료. HWM512MiB급→70.8MiB(약86.2%감소), RSS66.8MiB 표본. [원시 근거와 미검증](../validation/cpu-report-stress-validation.md)을 따른다. 페이지 계산 unit을 OS/GUI 성능 성공으로 대체하지 않았다. 미검증 다른OS/장시간/전체225page순회는 후속 릴리스 게이트이며 이 수직 Linux 회귀 작업의 완료와 구분한다.
