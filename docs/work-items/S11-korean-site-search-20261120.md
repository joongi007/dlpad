# S11: 한국어 소개·사용 안내와 검색 정보 정리

- 상태: in-progress
- 분류: improvement
- 우선순위: P1
- 요구사항: R10/R11 (문서·소개 페이지 운영), 사용자 한국어 안내·검색 최적화 요청
- 담당/수정 범위: 주 담당; site/index.html, site/styles.css, site/app.js, site/sitemap.xml, site/assets/favicon.png, scripts/build_site.py, scripts/test_build_site.py, scripts/test_site_browser.cjs, README.md, docs/CICD.md, 이 작업 기록
- 선행 작업: 기존 Pages 소개 페이지와 Google 소유권 태그 게시

## 사용자 결과

앱이 한국어 단일 UI임을 확인하고 소개·사용 안내를 한국어로 제공한다. dlpad와 yt-dlp GUI의 용도를 제목·본문에 자연스럽게 드러내고 검색·공유용 정보를 정리한다.

## 계약과 범위

기존 GTMUX 기반 사이트 토큰·폰트·실제 앱 스크린샷을 재사용한다. 도메인/호스트 설정, 다른 저장소, Google 계정, 앱 UI 언어 기능은 변경하지 않는다. 재생성 가능한 검수 출력은 /tmp에 둔다. 제품 의존성과 폰트 다운로드는 추가하지 않는다. 키워드 나열이나 허위 평점·지원 기능을 넣지 않는다.

Google은 favicon을 hostname별 하나만 지원하며 /dlpad/ 하위 경로만으로 별도 검색 favicon을 보장할 수 없다. PNG 정사각 아이콘과 sitemap을 배포하되 호스트 루트/전용 도메인은 이번 범위 밖으로 구분한다.

## 인수 조건

한국어 소개·설치/첫 사용 순서·언어 FAQ를 읽을 수 있다. title/description/canonical/공유 정보/사이트맵이 같은 실제 페이지를 가리키고 기존 확인 태그와 다운로드 링크 검증을 유지한다. PNG가 정사각이고 48px보다 크며 빌드 allowlist에 포함된다. 모바일/Light/Dark/키보드/실패 상태를 headless 브라우저로 검사하고 캡처를 직접 확인한다. Pages 게시 후 HTML/아이콘/사이트맵을 확인한다. 검색 순위나 실제 Google 재색인을 완료했다고 주장하지 않는다.

## 검증과 handoff

작업 중. 초기 Git 상태에 기존 수정이 있으므로 변경 파일만 별도 정상 게시 worktree에서 통합한다. 앱의 catalog.rs/installer_catalog.rs는 한국어 단일 문자열 catalog이며 별도 UI 언어 선택을 확인하지 못했다.


### 로컬 구현·검증

- 기존 catalog.rs/installer_catalog.rs와 설정 경로를 확인했다. 현재 UI 언어 전환·다국어 catalog 선택은 없으므로 한국어 소개와 안내를 유지했다. README의 첫 설명도 한국어로 바꿨다.
- title/H1/description에 dlpad·한국어 yt-dlp GUI·유튜브/치지직 다운로드라는 실제 용도를 반영했다. GUI 설명과 설치/도구 준비/첫 다운로드 순서를 추가했다.
- canonical, Open Graph, 한국어 lang/locale, 단일 URL sitemap을 추가하고 Google 확인 태그를 유지했다. 키워드 나열·번역 URL·허위 앱 평점을 추가하지 않았다.
- SVG 원본을 격리 headless Chromium으로 렌더한 PNG192×192는 2,038 bytes, sitemap.xml은169 bytes다. 빌드 allowlist는 release.json 포함9파일이며 제품 런타임 의존성·폰트·Rust 변경은 없다. 검수 라이브러리 playwright-core1.50.1은 기존 캐시 Chromium1155에 맞춰 /tmp에만 설치했다. 공개 사이트는 이 라이브러리에 의존하지 않는다.
- python3 scripts/test_build_site.py: 실제 빌드의 canonical/한국어/확인 태그/sitemap/PNG 검사를 포함해9 tests PASS. 기존 경계·실패 검사 유지.
- python3 scripts/check_docs.py --source-only: 1,151 documents PASS. 원 검증 자료는 검사하지 않았다.
- headless test_site_browser.cjs: 320/360/768/1440폭·Light/Dark·확대 viewport·키보드/포커스·hover·FAQ·정상/잘못된URL/오프라인/크기초과 응답 등11 scenarios PASS. 실제 SVG→PNG decode도192×192를 확인했다. 서버는127.0.0.1에만 바인딩했고 브라우저/서버가 종료됐다.
- /tmp/dlpad-site-review-20261120/captures-final의 실제 desktop/mobile Light 및 desktop Dark PNG를 주 담당이 직접 확인했다. 모바일의 쉼표 뒤 공백 누락과 한글 단어 중간 줄바꿈을 보완했다. 사용자 데스크톱이나 물리 모바일 검수로 확대하지 않는다.

### 검색 아이콘 제한과 남은 운영 결정

2026-11-20 공개 호스트 루트 https://joongi007.github.io/ 는 HTTP404였다. Google 공식 지침은 hostname별 하나의 favicon과 crawl 가능한 홈페이지를 요구한다. 따라서 /dlpad/의 PNG 교체 및 재수집만으로 Google 아이콘 표시를 보장할 수 없다. 전용 도메인 또는 호스트 루트의 사이트/아이콘 정책은 사용자가 결정할 별도 범위다. 다른 저장소나 루트 사이트는 변경하지 않았다.

재수집에는 며칠에서 몇 주가 걸릴 수 있고 표시 자체가 보장되지는 않는다. Search Console의 URL 검사/색인 요청과 https://joongi007.github.io/dlpad/sitemap.xml 제출은 운영자가 수행한다. [Google favicon 기준](https://developers.google.com/search/docs/appearance/favicon-in-search), [제목 지침](https://developers.google.com/search/docs/appearance/title-link).

공개 게시 검증은 진행 중이다.
