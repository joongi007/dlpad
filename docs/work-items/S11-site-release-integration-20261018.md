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


## 최신 통합 완료 범위와 공개 대기

c5963b5의 CI39066254029 전체 success를 Root가 GitHub 원 run 응답에서도 직접 확인했다. 앱 code는 origin/main에 normal FF push했으며 원격 refs/heads/main=c5963b5를 다시 확인했다. Root local main/index는 기존 상태를 보존하고 건강한 site-release worktree/독립 index에서 수행했다. Pages 변수 조회 권한과 실제 공개 비활성을 먼저 확인했고 tag/Release/Pages 설정/운영키 수정0이다. 후속 문서 전용 갱신은 동일 앱 code의 검증 결과를 기록한다.

받은 원 preview artifact12113907821은11,887,524B/SHA2568b695992806df2e5a5bf11b77144a6ec2e5bcc9c1492bbdc64dc710b1117bf23이다. installer ZIP SHA2566af3ab081610f121afff5e017561c61c2d4dfe5aa8687691fc59547e67f83f57, portable2f276191fe167adbbdf636aa7c6b765e7e09d429d4aace83af0ccd6cd8b110eb. Root exact7/6 namespace·manifest5·CRC/SHA/binding 일치를 확인했다. package-review.json의 당시 private_installation_pending 사실을 덮어쓰지 않고 후속 root-review.json을 따로 작성했다.

최신 실제 설치 관측은 .local-tools/update-integration-20261018/hosted-c596/private-install-c596-v1/이며 raw status REVIEW_REQUIRED를 유지했다. Root가 installed.png를 열고 실제 installed namespace8(5payload+install-ready/registration/seal)·journal5 SHA·foreign sentinel 보존을 직접 확인했다. registered record 이름의 Root 일회성 검사 가정 오류는 제품 registration.rs의 실제 계약으로 정정했으며 제품 수정은 없었다. 원 설치기/CPU hash와 자연종료0/원Job0/강제정리0/private desktop 폐쇄/입력 desktop 불변을 확인했다. 설치기 전체 인증/물리IME/clean-machine/Authenticode를 완료한 것은 아니다.

실제 설치 CPU(a03678047d6990fa04e303a4d269aa8f2c3fcd74631398bac55bc512571122b6)의 GUI는 gui-hosted-c596-v1/에 있다. Root는 actual Light960·200% focus-scroll·Dark restart PNG와 두 원 JSON을 직접 검수했다. 두 세션 자연종료0/원Job0/강제정리0/input desktop 불변이며 실제 공개 버전 없음 결과를 관측했다. OS theme 자체/물리 키보드/자동 설치 인수는 false로 보존한다.

동일 최신 ZIP은 native-hosted-preparation-c596-v1에서 원 SHA를 고정한 test-only candidate로 실제 새 bundle/journal 준비1PASS/0FAIL(13.37초)을 확인했다. 운영키/실제 새 프로세스 전환/프로필 rollback은 미완료다. 최신 검수 ZIP 경로는 .local-tools/update-integration-20261018/hosted-c596/packages/다. 소개 페이지는 site-preview-20261018-v3/index.html 및 site-review-20261018-v4/ 실제 캡처로 검수할 수 있다. 공개 배포 질문에 대한 사용자 답변은 아직 없어 게시하지 않는다.


## 2026-10-18 공개 인수 완료

사용자가 GitHub 공개 진행을 명시 승인했다. [소개 페이지](https://joongi007.github.io/dlpad/)와 [Windows v0.1.0 preview](https://github.com/joongi007/dlpad/releases/tag/v0.1.0)를 게시했고 [태그 실행](https://github.com/joongi007/dlpad/actions/runs/39101008422)의 전체 검증·동일 실행 원본 패키징·draft 생성을 확인했다. Root는 원 artifact digest, installer7/portable6 member·manifest5·동일 payload, SHA256SUMS·source/run binding을 직접 대조한 뒤 preview로 공개했다. 공개 asset4개를 인증 없이 다시 스트리밍해 검수 SHA와 비교했다. 수동 relay 회귀로 요청한 [main Pages 배포](https://github.com/joongi007/dlpad/actions/runs/39103421782)와 live release.json의 preview/코드 서명 없음·정확한 ZIP 링크, 실제 headless PC·모바일 화면을 확인했다. [공개 작업의 원 증거와 제한](S11-github-publication-20261018.md)을 따른다. 운영 서명·실행 중 앱 교체·rollback·깨끗한 실기기 인수는 이 완료 범위가 아니다.
