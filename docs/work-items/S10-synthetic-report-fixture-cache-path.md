# S10: 합성 진단 fixture의 쿠키 cache 경로 일치

- 상태: in-progress
- 분류: fix
- 우선순위: P0
- 요구사항: R13, R09, R11
- 담당/수정 범위: root 검토·통합; diagnostic_cache의 `/tmp` collector 수정안; scripts/windows_s10_report_save_probe.cs 및 본 문서
- 선행 작업: S10 Windows 격리 보고서 저장 시험

## 사용자 결과

진단 인수 시험이 합성 도구의 fixture 거절이 아니라 의도한 metadata 실패로 실제 보고서를 생성한다.

## 계약과 범위

제품 인증 정책이나 합성 도구의 경로 검증을 완화하지 않는다. 새 fixture에서만 앱 데이터 위치를 도구가 허용하는 root/data로 지정한다. root는 수집기 변경을 SHA 대조 후 통합하며 실제 계정·사용자 Documents를 사용하지 않는다.

## 인수 조건

합성 calls.jsonl에서 version 뒤 metadata 기록을 확인하고 fixture가 의도한 metadata 실패 exit code를 실제 DB/보고서와 대조한다. JSON/Markdown 저장과 정제 여부는 별도로 검증한다. 이전 exit64 fixture 거절 결과는 성공적인 metadata 실패 인수로 확대하지 않는다.

## 검증과 handoff

root 정적 대조: collector는 root/private-cache를 DLPAD_TEST_DATA로 지정했다. 앱 accessible.rs는 data/session-cookies를 사용한다. session_cookie_fixture_tool.rs는 cache를 root/private-cache 또는 root/data/session-cookies로 제한한다. 실제 조합 root/private-cache/session-cookies는 둘 다 아니므로 거절된다. 이전 저장 보고서의 Metadata exit64와 version-only 호출 기록에 부합한다. 수정안과 fresh native 실행은 진행 중이며 완료를 주장하지 않는다. 문서 검사 직전 상태는649개 PASS다.

root가 staged C# SHA `9c4c2962213d26c4a4dca1f0c5972c6ea3f02131b006dc7ea20b6a02b11856b4`와 runner SHA `59f8158471a954cfbf6c3575df3d15dcf32aa3c3363f9cd4eef5db03200e1566` 및 기존 원본 SHA를 대조한 뒤 두 project scripts에 통합했다. `windows-s10-save-scenario-20261004T133252Z`는 C# 실제 compile 성공 후 private GUI save scenario를 실행 중이다. 이전 overload 인수 초안 오류도 통합 전에 수정했다. 실제 stage/exit·calls·cookie SHA assertion은 해당 실행의 종료 및 root DB 검토를 기다린다.
