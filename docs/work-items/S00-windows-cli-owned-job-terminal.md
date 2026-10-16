# S00: Windows 전용 CLI 원 Job terminal 검증

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R03, R06, R10
- 담당/수정 범위: diagnostic_cache는 /tmp 새 host 초안·회귀 설계만, root는 scripts/windows_cli_host.cs와 원 owned tool host 게시·원 native 실행·통합 문서. Cargo/schema 변경 없음.
- 선행 작업: 현재 GUI/미디어 원 native 실행 terminal 후 공유 host/source 변경.

## 사용자 결과

시험 CLI가 direct child exit0을 보고해도 원 Job 안의 잔류 손자 프로세스를 성공으로 오인하지 않는다. 원 Job0·privateDesktop close·input desktop 보존을 구분해 기록한다.

## 계약과 범위

원 Job/handle만 조회·정리하며 기존 kill-on-close와 stdin cancellation·timeout을 유지한다. 공개/사용자 desktop·글로벌 input·프로세스 이름 기반 정리 없음. source 고정 상태의 root 게시/컴파일만 허용한다. std C# 기존 framework 사용, 제품 dependency/크기 증가 없음.

## 인수 조건

실제 원 directchild 정상 종료/원 Job natural0은 exit0; 원parent만exit0하고ownedgrandchild잔류는bounded기한뒤실패와소유트리정리. timeout·stdin취소·Job조회실패·privateDesktop close실패를성공으로기록하지않는다. 실제 원 cleanup 원로그/source/binarySHA와sourcefreeze 증거.

## 검증과 handoff

기존mediahost로그는 directchildexit0/private/inputDesktop 보존만 있고 명시Job0 receipt 없음. 새검증 전 기존결과를전체Job0확인으로확대하지않는다.

root diff검토후 새host를기존host미교체상태로정상/잔류손자/timeout/stdin4원native회귀를실행했다. 정상exit0/naturalJob0,잔류손자125/forcedcleanupJob0,timeout124/Job0,stdin130/Job0와원desktopclosed/inputsame를원terminal로그에서대조했다. 증거 windows-cli-job-terminal-first-20261005-resume/result.json. 모든실행terminal뒤원sharedhost/source를projectownedarchive에보존하고시험한exactsource/exe를게시했다. 새sharedhost로최신default전체검증/원native재시험은후속이다. syscall실패fault-injection은미실행이다.

실제Win32 fault3case(query -1 HANDLE/desktopClose -1 HANDLE/missingEXE)는원faultvariant125와failure receipt로거절됐고원outerhost는naturalJob0/privateclose/inputsame를기록했다. variants는test-only이며productionhost에게시하지않았다. root는원실패명령/n.run passedfalse를보존하며실패거절회귀true와구분했다. 증거 windows-cli-job-terminal-failures-20261005-resume/result.json. 재현용scripts/windows_cli_terminal_failure_probe.py만게시한다.

## 2026-10-05 원 결과 통합 완료

최신 Windows default CPU fmt/clippy/test/release 네 명령 exit0, 현재 소스534개 SHA 불일치0, CPU966c6b798c95113b9f0a191bdb22504742f51efcc030840dce7297bfe308759e/7856640B를 root가 직접 확인했다. 증거 [최신 전체 검증](../validation/evidence/windows-report-cap-current-default-20261005-resume/result.json). 이 작업의 좁은 인수 완료이며 전체 P0 완료가 아니다.

원host4정상/손자잔류/timeout/stdin과 test-only Win32 syscall3실패 거절 PASS. 최신 네 명령의 explicit natural Job0/finalJob0/privateDesktop-close/inputSame 원로그도 독립 확인했다. 실제 실패 n.run passed=false와 거절회귀 성공을 구분하며 host variant를 제품으로 게시하지 않았다. 제품 런타임 dependency 증가0.
