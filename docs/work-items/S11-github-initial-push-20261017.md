# S11: dlpad GitHub 초기 연결과 CI 실행

- 상태: in-progress
- 분류: maintenance
- 우선순위: P0
- 요구사항: R10, R11
- 담당/수정 범위: Root — 기존 빈 .git의 초기화와 local config/index/commit/remote/push, .gitignore, README와 CI/개발 안내·tracker 문서, 필요한 문서 검사·테스트, scripts/export_source.py의 .gitattributes export 허용; p0_audit — 읽기 전용 CI 초기 push blocker 검토
- 선행 작업: v37 검수본; 사용자가 https://github.com/joongi007/dlpad.git 연결·기존 인증 사용·Git 업로드를 명시 요청

## 사용자 결과

실제 작업 폴더를 Git 저장소로 연결하고 검토한 소스·CI 설정을 main에 올린다. 원격 Actions 결과와 설정 파일 존재를 구분한다.

## 계약과 범위

기존 .git은 HEAD/config/objects/refs 없는 빈 디렉터리임을 확인했고 삭제하지 않는다. 이번 명시적 Git 연결 요청에 따라 그 디렉터리를 초기화한다. 원격 ls-remote는 실제 exit0/refs없음으로 비어 있는 저장소다. Windows Git의 기존 작성자·인증을 사용하며 credential/token 자체를 출력하거나 다른 서비스로 보내지 않는다. 로그인 UI를 자동으로 띄우지 않는다. push는 force 없이 수행하며 원격 이력 충돌은 보존한다.

.local-tools·target·.aws/.agents/.codex·실제 사용자 데이터·로컬 실행 증거/스크린샷·ZIP을 소스 커밋에 넣지 않는다. 인증값·대형파일·기계 경로와 미포함 증거 링크를 점검하며 빌드에 필요한 native assets·vendor·lock은 유지한다. 로컬 증거는 삭제하지 않는다. CI/CD 설정과 공개 릴리스 배포는 별개다. 실제 공개 릴리스·서명·비용 자원·app updater는 이번 범위 밖이다.

## 인수 조건

- git status/top-level/remote/main이 실제 dlpad를 가리킨다.
- 커밋 파일 목록에 credentials·도구 cache·설치 산출물·원시 로컬 증거가 없고 제품 source675·lock/native assets가 보존된다.
- 소스 checkout의 문서·CI 정책 검사가 통과하며 미포함 runtime 증거를 검증 완료로 바꾸지 않는다.
- main 원격 commit과 push 결과, Actions 조회 가능 여부·실제 결과를 기록한다.

## 검증과 handoff

Windows Git ls-remote 실제0/empty, 작성자 설정 조회0. 최초 ls-remote 자동 승인 검토는 deadline timeout으로 완료되지 않았으며 허용된 한 번 재시도는 승인·성공했다. 승인 거절을 우회하지 않았다. 기존 사용자 자료 변경0, 공개 release0.


## 초기 연결과 checkout 정책

실제 .git은 기존 빈 디렉터리를 그대로 Windows Git init -b main으로 초기화했고 origin 추가·local core.autocrlf=false/core.filemode=false를 설정했다. 전역 config 변경은 없다. Windows Git user.name/user.email 기존 설정을 사용한다.

로컬 증거를 제외하면 strict 문서 검사에 missing 링크가 생기는 문제를 CI의 명시 source-only 모드로 처리했다. 일반 문서·source·native assets 누락/밖으로 나가는 경로/필수문서/toolchain 오류는 계속 실패한다. 선택모드의 local-only 링크만 export_source.omitted 정책으로 제외하고 NOT validated로 출력한다. 7개 실패경로 회귀를 추가했고 실제 통과했다. 기본 strict local 검사와 원 증거·기록은 유지한다. .gitattributes는 source/notice/resource의 원 bytes를 Windows clone에서도 보존하며 exporter allowlist에 포함했다. GitHub README는 소스 checkout·빌드와 아직 없는 정식 릴리스를 구분한다.


## 커밋 전 검사

Git 제외 정책 적용 후 예비 목록2142파일/25,112,504바이트, 최대파일710,285바이트로 CI tracked bound 이내였다. GitHub token/AWS access key/private key 형태 검사 위치0. .local-tools/계정 디렉터리/원시 evidence/실행 ZIP은 포함0, 빌드용 .ico/.res/.o는 포함했다. 예비 목록 수치는 문서 마감 전 snapshot이며 최종 index와 다를 수 있다. workflow docs job의19명령은 로컬에서 모두 exit0이며 제품 source675는 whole37과 byte동일하다. 이 변경 때문에 Rust 전체 빌드를 반복하지 않는다. clean index checkout도 검증 후 push한다.


## 실제 clean checkout 인수

Windows Git으로 준비한 index2142파일/25,113,997바이트를 읽어 모든 blob의 원본 bytes/SHA1과100644 regular mode를 대조했다. 빌드 resource 5개·Cargo/lock/toolchain/.gitattributes 포함을 확인했다. git checkout-index로 새 /tmp 폴더에 소스만 추출했고 그 checkout에서 workflow docs job의19개 Python 명령이 실제 모두 exit0이었다. 로컬 증거·도구·계정 상태를 복사하지 않았다. 원 작업 폴더의 strict `python3 scripts/check_docs.py`도1210documents/133hash-matched notices PASS다. 최종 이 문단 추가로 파일 bytes 합은 소폭 증가하며 새 source checkout 정책·제품 runtime 변경은 없다.

이제 초기 commit/push와 원격 Actions 수신을 확인한다. 원격 실행 성공을 로컬19명령 성공으로 대체하지 않으며 서명·CD·app updater는 미완료다.
