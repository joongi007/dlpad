# S11: 소개 페이지와 배포 진입점

- 상태: in-progress
- 분류: feature
- 우선순위: P1
- 요구사항: R08, R10, R11
- 담당/수정 범위: Root — site/, 사이트 build/검증 scripts, .github/workflows/pages.yml, README·CICD·ROADMAP와 본 문서. 공유 CI/Cargo/lock은 Root 조정.
- 선행 작업: 실제 Windows CI run39040643416 전체 성공; 사용자 2026-10-18 소개 페이지→자동 릴리스→앱 업데이트 순서 지정

## 사용자 결과

한국어 제품 소개, 실제 안전한 화면 이미지, 사용 방법과 공식 다운로드 진입점을 갖춘 정적 페이지를 제공한다. Windows 설치 ZIP과 포터블 ZIP을 공개 Release와 연결한다. 앱 UI는 기존 Rust 네이티브를 유지한다.

## 계약과 범위

소개 사이트는 제품 UI/localhost 제품 서버가 아닌 별도 정적 배포물이다. 브랜드는 기존 앱의 차분한 회색·파랑과 시스템 한글 타이포그래피를 잇는다. 물리 장면: 다른 작업 중 Windows 앱을 찾는 사람이 낮의 브라우저와 밤의 노트북에서 기능과 다운로드를 빠르게 확인한다. Light 기본·System 반영으로 시작하고 수동 테마/키보드 접근·모바일을 검증한다. 자체 asset과 개인 자료가 없는 실제 fixture 화면만 사용하며 숫자·지원 OS·업데이트 제공 상태를 과장하지 않는다.

릴리스 유무/실패/오프라인 때도 명확한 안내와 GitHub Releases 링크가 있어야 한다. 임의 asset이나 draft/prerelease를 stable 최신판으로 표시하지 않는다. 기본 웹 UI 의존성/서버 런타임0, 모델·다운로드 도구 payload 포함0. 실제 게시 전 사이트와 package 후보를 구체적으로 검증한다.

## 인수 조건

- 360/768/1440px와 키보드·200% 확대·Light/Dark에서 잘림/버튼 상태/콘텐츠 위계·다운로드 링크를 실제 브라우저로 확인한다.
- 최소 정적 산출물만 공개하고 개인 경로·쿠키·raw evidence·cache는 포함하지 않는다.
- source-only/default 문서 검사·사이트 링크/릴리스 처리 실패 경로·build와 배포 workflow 검증을 기록한다.

## 검증과 handoff

문서와 기존 배포 계약을 읽고 구현 중이다. 공개 사이트/릴리스/앱 updater 완료는 아직 아니다. 다음 순서는 자동 Windows 릴리스와 서명된 앱 업데이트다.

## 2026-10-18 첫 구현과 실제 브라우저 검증

`site/index.html`, `styles.css`, `app.js`, 자체 SVG 아이콘, 실제 Windows 앱 PNG, 공개 release.json을 구현했다. 시스템 글꼴/Light·Dark/반응형/키보드 focus/hover/FAQ와 설치형·포터블 진입점을 제공한다. 버전은 공개 Release에서만 가져오며 초안·불완전 asset 쌍·다른 저장소 URL은 다운로드 후보가 아니다. Python stdlib allowlist build 실패회귀8개 PASS. 페이지 runtime dependency0, 외부 폰트/분석/visitor GitHub API0. 공개 산출물7개이며 그림 원본42,032B다.

실제 headless Chromium131(Playwright-core1.50.0, 기존 개발 도구)을 통해360/768/1440px, Light/Dark,720px 확대 등가 viewport, Tab·본문 건너뛰기·Enter FAQ, 테마 전환·hover, 정상/불법URL/오프라인/과대 release.json 총10시나리오 PASS. 첫 키보드 시험에서 main focus 누락을 발견해 tabindex=-1로 수정 후 재검증했다. 720px 시험은200% 실제 OS/browser zoom 인증이 아닌1440px의 layout 등가 viewport다. 200% 실브라우저 확대와 OS 접근성 인증은 완료로 주장하지 않는다.

실행: `python3 scripts/test_build_site.py`; `python3 scripts/build_site.py --output /mnt/d/project/dlpad/.local-tools/site-preview-20261018-v2`; `FONTCONFIG_FILE=/tmp/dlpad-site-fonts-20261018/fonts.conf node scripts/test_site_browser.cjs .local-tools/site-preview-20261018-v2 /mnt/d/project/dlpad/.local-tools/site-review-20261018-v3 /home/joongi/.npm/_npx/4de92097080e5d0d/node_modules/playwright-core /home/joongi/.cache/ms-playwright/chromium-1155/chrome-linux/chrome`. 개발 환경 도구 경로는 실행 기록이며 프로젝트 실행 의존성이 아니다. 시험 서버127.0.0.1과 headless browser는 정상 종료, 사용자 desktop GUI0. sandbox의 첫 listen EPERM은 허용된 loopback 범위의 정식 escalation으로 실행했으며 거절 우회0.

Root가 desktop-light와 mobile-dark PNG를 실제 열어 한글·배치·잘림을 확인했다. PNG 원본은 `docs/validation/evidence/windows-msvc-ui-feedback-v32-install-reinstall-launch-private-v1-20261016/gui/installed-app-private.png`다. 개발 소유 경로만 있고 개인 사용자명/계정/영상/쿠키가 없으며 캡처를 편집하거나 최신 UI 재촬영으로 표시하지 않았다. 새 제품 GUI 검증을 대신하지 않는다.

Pages Actions의 공식 SHA refs API를 확인했다. 자동 build/artifact는 구성했고 공개 deploy는 DLPAD_PAGES_ENABLED 명시 설정 뒤에만 실행한다. 실제 hosted 실행/Pages 게시/첫 공개 Release는 미검증이다.
