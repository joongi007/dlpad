# S11: 소개 페이지 Google 소유권 확인

- 상태: done
- 분류: maintenance
- 우선순위: P1
- 요구사항: R11 (소개 페이지 배포 운영)
- 담당/수정 범위: 주 담당; site/index.html, 이 작업 기록
- 선행 작업: 기존 GitHub Pages 소개 페이지

## 사용자 결과

사용자가 제공한 Google Search Console 소유권 확인 태그를 소개 페이지의 head에 게시한다.

## 계약과 범위

기존 정적 사이트 빌드와 Pages 배포를 유지한다. 앱 UI, 런타임 의존성, 분석 스크립트는 추가하지 않는다. 공개 HTML에 넣도록 제공된 확인 문자열을 그대로 사용한다.

## 인수 조건

소스와 사이트 빌드 결과의 head에 정확한 태그가 한 번 포함된다. 기존 사이트 검사와 문서 검사를 통과한다. 공개 배포 반영 여부와 Search Console에서 사용자가 수행할 소유권 확인을 구분해 기록한다.

## 검증과 handoff

원 작업 폴더의 기존 수정은 보존했다. 변경 파일은 site/index.html과 이 작업 기록이다. Rust 코드·도구·의존성은 변경하지 않았다.

실제 실행 및 결과:

- `python3 scripts/test_build_site.py`: 8 tests PASS.
- `python3 scripts/check_docs.py --source-only`: 1,150 documents PASS. 로컬 원 검증 자료는 검사 범위 밖이다.
- 임시 폴더에서 build_site.build 실행 후 HTMLParser로 결과 확인: head 안에 제공된 확인 문자열이 정확히 한 번 존재함을 확인했다. 임시 출력은 자동 정리했다.
- 정상 Windows Git 작업본에서 변경 두 파일만 커밋하고 main에 fast-forward push했다. 소스 커밋 `1e9717e7e8003159a21c859a72dc0d814f80f5f6`.
- [Pages 배포](https://github.com/joongi007/dlpad/actions/runs/43630448184): completed / success, 위 커밋과 일치.
- 2026-11-15 04:09 KST, [공개 페이지](https://joongi007.github.io/dlpad/) HTTP200 원 HTML의 head에 정확한 태그가 한 번 존재함을 독립 파싱으로 확인했다.

Google 계정의 소유권 확인 버튼과 색인 요청은 사용자가 Search Console에서 수행한다. 이 변경은 태그 게시를 완료한 것이며 Google의 검색 색인 등록·순위·노출을 검증한 것이 아니다. 태그는 확인 후에도 유지한다. [Google 소유권 확인 안내](https://support.google.com/webmasters/answer/9008080?hl=ko).
