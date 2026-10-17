# S11: 소개·릴리스·업데이트 소스 통합

- 상태: in-progress
- 분류: feature
- 우선순위: P1
- 요구사항: R08, R11
- 담당/수정 범위: Root 공유 Cargo/lock/모듈·native navigation/수명·.github/workflows·README/CONTEXT/CICD/ROADMAP/ADR index·작업기록·최종 검증. release_audit는 별도worktree의패키징/notice/signing, updater_audit는별도worktree의metadata/staging/discovery/panel/catalog, update_transport는/tmp전용HTTP. 파일 동시 수정0
- 선행 작업: source7819c9e의 정상Git/WindowsCI, 사용자 지정 소개→자동릴리스→업데이트 순서

## 사용자 결과

소개 페이지와 실제 설치형·포터블 candidate ZIP, native 버전조회와 서명/저장 기반을 같은 검증 소스로 통합한다. 공개와 실제자동교체 미완료를 명시한다.

## 계약과 범위

에이전트보고를 통과증거로 쓰지 않고 Root가 원Windows stdout/Job/sourcefacts·실제PNG·signer→Rust이종검증·실제hostedstatus를 읽는다. 공유Cargo는 기존ring0.17.14 직접참조1행만추가하며 새crate버전0이다. static사이트는제품UI가아니며 앱Rustnative계약을 유지한다. 운영키생성/외부전송/공개배포/태그/이력재작성은수행하지않는다.

## 인수 조건

Root fullWindows strictlint/test/release, privateUI한글/조회/배율/종료, 사이트headless실제10시나리오, 원패키지실패회귀·문서검사를 실행한다. sameSHA 원binary만패키징하고 잘못된candidate/tag·취소뒤draft·서명실패fallback을거절한다. 실제hostedZIP결과를소스설정과구분한다.

## 검증과 handoff

소개/릴리스 소스는 7b5d784 및32607f7로 원격main에normalpush했다. source32607f7의 site39057841523/CI39057841587 전체success다. RootWindows주1728+filteredchild1PASS/0FAIL/43ignored·strictlint·release와 privateGUI검수는 각 작업원기록에 있다. 공개Pages/Release·운영키·자동교체는미완료다.

로컬main은7819c9e기준으로이번변경을보유하며 .git/index.lock이읽기Git실행뒤에도재생성됐다. 당시 원프로세스없음/원파일exclusive열기만으로 소유를확인한빈lock1개를 .local-tools/update-integration-20261018/git-owned-query-lock-recovery에보존했으나 후속재생성의creator는미확인이다. 추가삭제/우회/기존index재구성없이 .local-tools/worktrees/site-release-20261018의 codex/site-release-20261018 branch/독립index에서선별commit·normalpush한다. 원main index·기존작업·기록을보존했다. 따라서 로컬mainHEAD가원격최신이라고가정하지말고이통합worktree와 origin/main을확인한다. root작업소스와통합worktree의파일hash를비교하고검증소스일치를확인한다.


소개/자동ZIP 검수 main은49c3812이며 CI39060610796 전체success와 실제원ZIP 설치/설치앱private실행까지 Root검토했다. 운영공개는사용자초기별도승인범위에따라질문을남겼고답변전실행하지않는다. 후속 read-only bundle검사는별도 codex/update-bundle-integration-20261018 branch에서 전체CI를확인하며 승인대상49c검수본과구분한다. 기존localmain/index/새helper/운영key/공개site 상태를혼동하지않는다.


후속 ab4a7c1 CI39064076669는 기존 archive_install notice 읽기 시험의 단계 오인으로 Tests에서 실패했다. 원 로그618PASS/1FAIL/17ignored/exit101을 보존했고 후속 빌드/GPU/autoZIP는 skipped였다. [정확 재현과 최소 수정](S11-ci-notice-read-deadline-20261018.md)은 제품 코드/정책을 바꾸지 않는 fixture 수정이다. 별도 설치 준비와 함께 실제 Windows engine strict lint·전체 주635+filtered child1PASS를 통합 검증했으며 새로운 isolated integration branch에서 hosted workspace CI를 다시 확인한다. ab4실패를49c검수본/완료된 자동 공개로 오인하지 않는다.
