# S02: Windows 세션 쿠키 입력과 보존 안내

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: root — crates/app/src/accessible.rs, 필요시 catalog.rs, 이 문서 및 통합 문서
- 선행 작업: Windows cookie ACL runtime4, Job terminal3, interruptiblepipe actual시험

## 사용자 결과

Windows native 후보에서 선택한 Netscape 쿠키를이번worker만사용한다. 원본을보존하고private복사본을metadata/download에만전달한다. 인증 raw 출력·자유metadata는기존생략정책을유지한다. 이전보류복사본은삭제/재사용하지않고고정안내를표시하며새선택작업은가능하게한다.

## 계약과 범위

worker에서prepare_session_cache와import_session을사용하고UIthread에서긴I/O없음. 시작recoveryworker는기존cache가있는경우에만privateACL을검증한뒤최대1entry존재관측으로pending표시한다. Windowsrecover_sessions는Unavailable유지,receipt없이copy삭제없음. unsafecache는시작gate실패로보존한다. cookie선택은memtake로소비하고DB/옵션snapshot/로그에source/copy를저장하지않는다. 기본IcedUI/finalstack결정을이작업으로확대하지않는다.

## 인수 조건

actualWindowsCPUclippy/test/release,invalidsourcejob0/선택소비,existingdeferredprivatecopy보존+pending안내,unsafeACLcache시작차단,성공·취소auth작업copy정리/Job확인/secret없음. 실제GUI는privateDesktop에서만실행하고사용자desktop/clipboard/실제credential은사용하지않는다.

## 검증과 handoff

engineactualWindowscookie4/Job3는동결source시험을통과했다. pipe연결시험후UI연결을진행한다. Windowsstartup회수·실제사이트·최종UI/전체P0완료는남은gate다.

## root Windows 후보 연결 첫 검증

[actualWindowsCPU시험](../validation/evidence/windows-cookie-ui-first/root-review.json) session76442 exit0/34.39초, 입력before/after/current271SHA일치. 기존선택소비·잘못된sourcejob0와신규deferredcopy보존/pending안내·non-directorycache차단을실제로그에서확인했다. WindowsCPUall-targetClippy session55425 exit0/10.22초. 최초fmt검사는줄바꿈2곳으로exit1이었고원시범위를보존했으며cargo fmt뒤fmtcheck session41156 exit0. 논리수정은없고후속release는정리된소스로검증한다. 실제성공authenginefixture·privateDesktopGUI·Windowsstartup회수/전체P0는미완료다.
