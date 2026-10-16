# S10: 최신 제품 진단 복사 검증

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R13, S10
- 담당/수정 범위: 주 담당; 본 문서, 전용 합성 진단 evidence
- 선행 작업: Linux 진단 catalog/export UI 및 최신 source 통합

## 사용자 결과

정제 보고서를 미리보고 명시적으로 복사하는 오프라인 기본 진단 흐름을 최신 제품에서 재확인한다.

## 계약과 범위

기존 native_probe.py read-only. private Xvfb의 별도 clipboard·합성 fixture만 사용하며 사용자 clipboard/실제계정/외부전송 없음. 제품소스와 Windows snapshot 수정 없음.

## 인수 조건

최신 release 해시·보고서 정제 sentinel·private clipboard 실제 내용을 확인한다. 다른OS clipboard/전체 omission UI와 실제 reader는 별도 게이트다.

## 검증과 handoff

`DLPAD_TEST_FONT=/mnt/c/Windows/Fonts/malgun.ttf python3 scripts/native_probe.py target/release/dlpad docs/validation/evidence/S10-current-clipboard.json --page diagnostics --copy-report --seconds 5` exit0. 최신 release SHA256 `7e5fe81444a31806189e3a84d51d1ccf809f48bd3c83df1396c9d2fbabacd469`, isolated X11 selection 실제 UTF8 보고서2957B/headline 확인, fixture host와signature sentinel 미포함, owned probe reaped. [실제 증거](../validation/evidence/S10-current-clipboard.json). Linux copy만 실증했으며 save/omission 전체UI/다른OS를 이 결과로 완료하지 않는다.
