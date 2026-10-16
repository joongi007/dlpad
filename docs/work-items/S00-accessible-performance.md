# S00: 접근성 다운로드 후보의 대기 비용

- 상태: done (measurement; proposed memory budget FAIL)
- 분류: verification
- 우선순위: P0
- 요구사항: R08, S00
- 담당/수정 범위: 주 담당; scripts/accessible_performance_probe.py, 전용 validation/evidence, 본 문서와 병렬 보드
- 선행 작업: frozen dlpad-accessible release 및 실제 worker 연결

## 사용자 결과

접근성 개선 후보가 대기 중 반복 redraw로 CPU를 소모하지 않는지 실제 native 프로세스에서 확인한다.

## 계약과 범위

제품/Cargo/schema 변경 없음. 기존 native_probe display/screenshot/sample은 read-only 사용한다. owned Xvfb/private D-Bus/temp XDG/appdata만 사용하며 사용자 화면·Documents·실제영상·기존 캐시를 사용하지 않는다. 로그 각64KiB 상한, 10초 표본과 owned cleanup, streaming binary hash를 기록한다.

## 인수 조건

실제 release hash와 프로세스 identity, RSS/CPU 10초 표본 및 screenshot을 남긴다. 대기 비용을 실제다운로드/10,000목록/8시간/다른OS/최초프레임으로 확대하지 않는다. 제안80MiB budget을 넘으면 실패를 기록하고 최종stack채택을 보류한다.

## 검증과 handoff

작성 후 문법검사와 실제 격리 실행. 결과 전 성공으로 쓰지 않는다.

실제3회 측정 exit0/cleanup완료. 기본196156KiB, llvmpipe138204KiB, softpipe247564KiB이며 idle CPU10초표본0%. 80MiB제안 budget 모두 FAIL. [실제 결과](../validation/accessible-performance-validation.md)에 원시JSON/조건/미검증범위를 기록했다. 비용측정 작업 완료이며 성능 개선/최종stack채택은 남는다.
