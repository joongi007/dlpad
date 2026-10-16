# S00: Windows 격리 실제 screen reader 실행 가능성

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R01, R08 및 PRODUCT/S00 접근성
- 담당/수정 범위: soak_recovery_audit는 /tmp/dlpad-private-reader-* 읽기 조사, root는 이 문서와 후속 구체 구현/통합. 기존파일/Cargo는root만수정한다.
- 선행 작업: privateWindowsGUI/UIA원provider; 실제NVDA/Narrator는미검증.

## 사용자 결과

실제reader에서제목·상태·버튼·키보드포커스읽기를증명할격리방법을확인한다. UIAutomation시험을reader시험으로재명명하지않는다.

## 계약과 범위

공식NVDA소스/문서의portable/configscope/전역flag/audio/hook/중복인스턴스정책과privateDesktop수명경계를읽기조사한다. 사용자desktop/Narratortoggle/기존reader설정변경/전역hook실행·새reader실행은사전근거없이하지않는다. 다운로드·네이티브실행은이번조사범위에없다. 외부게시/실계정/인증없다.

## 인수 조건

공식근거와구체argv/config/프로세스소유권/private지원및미확인조건을기록한다. 격리불가면실제VM/session필요조건을남기고독립개발을계속한다. 안전한실제reader인수는후속원speechlog·actualfocus·자연종료/Job0로별도판정한다.

## 검증과 handoff

읽기조사시작, 제품reader미완료. 최종스택/P0의접근성조건을축소하지않는다.

읽기조사범위완료. [공식근거와제약](../agents/handoffs/20261005/private-reader-feasibility.md)에portable/config경로/--no-sr-flag와별개EaseOfAccess.notify의registeredNVDA조건부registry쓰기/SendInput을기록했다. currentmaster를고정exe와동일하다고확정하지않으며원pinnedraw조회오류도기록했다. 이환경에서reader실행은하지않았다. 실제screen reader 인수는별도격리VM/session의입력·registry·audio경계확인후수행해야하며P0/S00미완료를유지한다. 사용자desktop에reader를켜거나등록삭제/전역동작을우회하지않는다.

2026-10-05 후속 readonly readiness: root는 scripts/windows_isolation_readiness.ps1과 원20초 private CLI host로 exact optional feature/Get-VM 능력 및 WTS session count만 조회한다. VM/session 시작·연결·screen reader·clipboard·system change는 수행하지 않는다. feature query 권한 부족은 미확인으로 기록하며 관리자 권한을 얻어 변경하지 않는다. 기존 path-only 감사는 /tmp 자료이며 실제 native 조회를 대체하지 않는다.

후속 readonly PS 조회는 원 ExecutionPolicy의 UnauthorizedAccess로 실행 전 거절됐다. [원 실패](../validation/evidence/windows-isolation-readiness-native-20261005-resume/readonly-capability.json). naturalJob0/privateDesktop/inputSame는 확인했으나 optional feature/VM/session 상태는 미확인이다. ExecutionPolicy 변경·Bypass·encoded command 우회는 수행하지 않았다. 제품 개발·합성실패검증은 독립 진행한다.
