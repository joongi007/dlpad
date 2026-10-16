# S10: Linux 진단 복사 실제 경로 검증

- 상태: done
- 분류: test
- 우선순위: P0
- 요구사항: R13, R08
- 담당/수정 범위: 주 담당; scripts/x11_diagnostics.py, scripts/native_probe.py, scripts/test_native_probe.py, 진단/검증 문서
- 선행 작업: S10 개발 UI와 정제 보고서, S00 isolated probe

## 사용자 결과

진단 복사 버튼이 실제 Linux clipboard에 정제된 보고서를 전달하는지 검증한다.

## 계약과 범위

합성 보고서와 별도 Xvfb display만 사용. Xtst로 해당 가상 화면 내부 버튼 클릭, X11 selection으로 2 MiB 이하 텍스트 확인. 실제 사용자 clipboard/desktop에는 접근하지 않는다. desktop 모드의 복사 exercise는 거절한다. Python/XTest는 개발 도구이며 제품 의존성 아님.

## 인수 조건

한국어 복사 버튼 클릭→정제 Markdown clipboard 수신→원본 URL/signature 부재. 프로세스 회수 및 headless command 실패 상태. macOS/Windows clipboard와 대형 INCR clipboard는 별도 검증.

## 검증과 handoff

Xvfb에서 한국어 복사 버튼 클릭 후 실제 X11 clipboard의 정제 Markdown 2,957 bytes 수신 확인. 원본 URL/signature 부재, 프로세스 회수 및 desktop 복사 시험 거절 검증. Windows/macOS와 대형 INCR clipboard는 미검증. 공통 fmt/clippy/test/release 통과. 전체 74 tests 및 실행 명령/측정은 [통합 검증](../validation/followup-validation.md)에 기록한다. 작업 범위 완료이며 S00 최종 스택 또는 전체 P0 완료를 뜻하지 않는다.
