# S11: 공개 릴리스의 Pages main 갱신 연결

- 상태: in-progress
- 분류: bug
- 우선순위: P1
- 요구사항: R10, R11
- 담당/수정 범위: 주 담당; .github/workflows/pages.yml, .github/workflows/pages-release.yml, docs/CICD.md, 본 문서
- 선행 작업: 실제 v0.1.0 공개, Pages workflow/main 배포 성공

## 사용자 결과

릴리스 공개 시 소개 페이지의 다운로드 정보를 main 소스에서 다시 빌드한다. 기존 Pages 환경의 main 전용 배포 규칙을 유지한다.

## 계약과 범위

실제 published run39102954680은 build 성공·deploy 실패다. annotation은 `Tag "v0.1.0" is not allowed to deploy to github-pages due to environment protection rules.`였다. 후보는 (1) release event의 tag ref와 main 전용 환경 불일치, (2) pages/id-token 권한 부족, (3) 사이트 생성 실패였고 실제 build 성공·deploy 시작 전 환경 annotation으로 (1)을 확인했다. diagnose skill의 관측→재현→가설→교정→실제 hosted 회귀 순서를 사용한다.

태그 소스를 직접 배포하거나 환경 보호 규칙·승인 정책을 바꾸지 않는다. 공개 이벤트를 받는 별도 workflow는 actions:write 권한의 단일 job으로 pages.yml의 main workflow_dispatch만 요청한다. 실제 사이트 빌드·배포는 main workflow의 기존 정책을 통과한다. Cargo·제품 UI·운영키·공개 Release 원파일을 변경하지 않는다.

## 인수 조건

- 원 실패의 source/run/annotation을 보존한다.
- main 사이트 workflow가 release ref를 직접 배포하지 않는다.
- relay를 실제 hosted workflow_dispatch로 실행하여 GITHUB_TOKEN의 main dispatch와 Pages 갱신을 검증한다.
- 실제 공개 release.json과 브라우저 링크가 v0.1.0 preview ZIP을 가리킨다.
- 현재 v0.1.0 태그는 변경하지 않고, 후속 공개 이벤트의 자동 연결 구성과 수동 relay 회귀를 구분한다.

## 검증과 handoff

진행 중. GitHub 공식 GITHUB_TOKEN 문서는 workflow_dispatch가 새 workflow 실행을 만드는 예외임을 명시한다. 기존 태그의 workflow는 변경하지 않는다. 실패 run을 성공으로 바꾸거나 첫 published 이벤트의 자동 배포가 성공했다고 기록하지 않는다.
