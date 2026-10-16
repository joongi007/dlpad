# S10 CPU 최대 유효 보고서 stress

- 상태: done
- 분류: verification
- 우선순위: P0
- 요구사항: R13
- 담당/수정 범위: guardian_fault; 신규 scripts/cpu_report_stress_probe.py, 이 문서 및 전용 validation/evidence
- 선행 작업: root 정확 frozen binary/실제 유효 report 경계 계약 합의

## 사용자 결과

실제 API로 구성 가능한 큰 정제 보고서의 preview/copy/create-only save와 기존 export 보존/원 job 불변/응답성을 격리 native GUI에서 검증한다.

## 계약과 범위

제품/Cargo/공유 helper 수정 금지. core logs128×4096B/MPD8×256nodes/1MiB입력/range48B/URL pseudonym 제한을 먼저 실제 API로 산출한다. 2MiB는 방어 상한이며 정확 경계가 실제 데이터로 생성 불가능하면 최대 유효 보고서로 범위를 명시한다. Boxleak unit을 GUI PASS로 대체하지 않는다. owned Xvfb/privatebus/합성 fixture만, user display/clipboard/media/network/auth 금지.

## 인수 조건

실제 유효 API 크기 산출과 제작 경로 확인. 합의한 최대 유효 fixture의 preview/copy/save/collision/originaljob/CloseWait를 원시 JSON/MD/SQLite/PNG로 보존. 2MiB 초과 방어 unit과 실제 GUI 최대유효범위를 분리한다. 실패는 미완료로 기록한다.

## 검증과 handoff

현재 read-only 한도 조사 중. 제품 fixture API 필요 여부는 root와 합의 전 변경하지 않는다. 기존8h 재시작·중단 금지.

## 재개 2026-10-03

담당 전용 harness만 수정하며 full-4~8 raw 실패 근거를 보존했다. 실제 footer geometry, AT-SPI button action, bounded widget extents trace 및 outer-margin wheel을 추가했다. 실제 Failed job/열린 header까지 확인했지만 copy가 content clip 밖에서 정지하여 최대보고서 인수 조건은 미완료다. root 공유 UI의 중첩 ScrollArea 높이 예약 확인에 의존한다. 자세한 근거는 [검증 기록](../validation/cpu-report-stress-validation.md)에 있다. 기존 WSL중단 full-3을 성공/소유프로세스 종료 증거로 표현하지 않는다.

## 기능 회귀 결과와 성능 후속

새 SHA2d6f64a…에서 full-10 기능 시험 exit0/PASS. 실제 visible controls를 통해 Markdown/JSON ~1.9MB 복사, 새 파일 저장·기존 bytes 보존, 원 Failed job/Attempt/options 불변, 정상 종료와 process 회수를 확인했다. full-9 harness inner-scroll 실패는 원시 보존. 메모리 high-water 약511.8MiB를 발견해 P0 성능 인수 완료로 보고하지 않는다. root가 공유 preview layout 개선을 담당하고 이 작업은 새 snapshot 회귀 검증까지 in-progress로 유지한다.

## 페이지화 후 인수 완료 (Linux 유효 fixture 범위)

full-10 큰 메모리 비용은 S10-cpu-report-preview-pages에서 화면만 UTF-8 8KiB 단위로 바꿔 해결했다. paged-3/4 실제GUI PASS: 동일 최대 유효 자료의 전체copy/export 보존/첫·끝·이전·다음페이지/기존 파일 보존/원job 불변/정상exit/소유 종료, HWM70.8MiB/RSS66.8MiB 표본. exact2MiB 생성이나 전체P0/다른OS 인수 완료가 아니다. [검증 근거](../validation/cpu-report-stress-validation.md)에 실패 보존, bytes 비교, 비용과 미검증 범위를 기록했다.
