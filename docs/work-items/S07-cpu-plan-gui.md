# S07: CPU 실행 계획 실제 GUI

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R07, R08, R11
- 담당/수정 범위: metrics_recovery — 신규 scripts/cpu_plan_probe.py 및 전용 evidence/문서
- 선행 작업: root preview/shared UI 연결, Linux frozen SHA166946017de0b3d0589bf3d02bd864d004d90f3457ff2e314fe5e10a5f865417

## 사용자 결과

다운로드를 실행하지 않고 입력 변경이 정제된 argv 미리보기에 반영되는지 확인한다.

## 계약과 범위

private Xvfb/DBus/temp roots/합성 실행파일만 사용한다. 시작 버튼을 누르지 않고 shared helpers/source/Cargo를 변경하지 않는다.

## 인수 조건

actual Text.GetText JSON에서 원본URL 생략, literal 경로·720품질·구간·고급argv 보존, 변경 즉시 반영, invalid 고정오류, job/attempt와 도구실행 없음, 작은 preview의 이전/다음 disabled 경계. 대형 page 및 다른OS는 별도 검증이다.

## 검증과 handoff

실제 실행 exit0/PASS, source/명령/raw links와 수집기 실패 원인은 docs/validation/cpu-plan-validation.md에 기록했다. actual JSON redaction/Unicode·구간·고급argv/즉시반영/invalid 오류/1page disabled 경계와 job·attempt 미실행을 확인했다. 종료0.154초/cleanuptrue/binary SHA 보존. 종료후source manifest를 기록했으며 전후source 대조라고 주장하지 않는다. 대형 page/256KiB 실패UI/다른OS/계획헤더 AT-SPI·키보드 전체 및 peak성능은 별도 검증이다.
