# S11: Pages 릴리스 조회의 무인증 요청 한도 제거

- 상태: in-progress
- 분류: fix
- 우선순위: P1
- 요구사항: R08, R10
- 담당/수정 범위: Root; .github/workflows/pages.yml, docs/CICD.md, 이 작업 기록
- 선행 작업: v0.1.2 공개 및 main 문서7d88b8b push

## 사용자 결과

소개 페이지의 다운로드 링크와 업데이트 안내가 GitHub 공유 runner의 무인증 API 한도 때문에 배포되지 않는 문제를 수정한다.

## 계약과 범위

실제 Pages run39142717714의 build 단계 원로그에서 HTTP403 rate limit exceeded를 확인했다. build_site.py 정책 테스트8개는 통과했고 기존 공개 페이지/릴리스는 보존되어 있다. workflow의 기존 contents:read GITHUB_TOKEN으로 github.com의 현재 저장소 releases API를 한 번 조회해 runner 임시 JSON으로 전달한다. 게시 대상은 기존 allowlist7파일뿐이며 입력 JSON·token·draft·비공개키는 게시하지 않는다. 앱/서명/부모/의존성 변경0. 무조건 재시도나 API한도 우회 대신 정규 인증된 서비스 요청을 사용한다.

## 인수 조건

정확 main의 hosted Pages build/deploy 성공, 공개 release.json의0.1.2 exact ZIP2링크 및 index.html 새FAQ 반영. draft 제외·응답 크기/URL 정책8회귀를 유지한다. 로컬 Chrome headless 및 가상앱 검수와 실제 hosted 배포를 구별한다.

## 검증과 handoff

수정 전 원로그는 ignored `.local-tools/update-publication-20261018/pages-docs-build-failure-v2.log`에 보존한다. 구현 후 로컬 site정책검사와 source-only 문서검사, 실제 Pages 배포/공개HTML 확인을 기록한다. 별도 운영서명키 GitHub공급·환경보호 변경은 하지 않는다.
