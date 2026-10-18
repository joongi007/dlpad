# S11: GitHub 소개 페이지와 Windows preview 공개

- 상태: done
- 분류: feature
- 우선순위: P1
- 요구사항: R10, R11
- 담당/수정 범위: 주 담당; GitHub Pages 설정·배포, v0.1.0 태그·Release, README.md, docs/CICD.md, S11-introduction-site-20261018.md, S11-auto-release-20261018.md, S11-site-release-integration-20261018.md, 이 작업 문서
- 선행 작업: S11-introduction-site-20261018, S11-auto-release-20261018, source c5963b5 Windows CI 및 격리 설치·GUI 검증

## 사용자 결과

GitHub Pages 소개 페이지에서 Windows 설치형·포터블 preview ZIP을 내려받는다. 태그 기반 검증·패키징과 미공개 draft 생성 이후 검수한 Release를 공개한다.

## 계약과 범위

2026-10-18 사용자의 “음 걍 진행해줘 그 공개가 어디에 공개한다는거야? 걍 github아니야?” 지시로 이 저장소의 Pages와 Release 공개를 승인받았다. 기존 원격 상태를 확인하고 변경하며 기존 태그·Release를 덮어쓰거나 Git 이력을 재작성하지 않는다. 공유 schema·Cargo 의존성·운영 서명키는 변경하지 않는다. 자동 앱 교체·rollback은 제외한다. unsigned preview를 서명된 정식 배포로 표현하지 않는다. ADR-0012는 Proposed 상태를 유지한다.

## 인수 조건

- 실제 Pages URL이 HTTPS로 열리고 릴리스 없을 때도 정직한 안내를 제공한다.
- 정확한 버전 태그와 원본 source SHA의 전체 release workflow가 성공한다.
- 공개 전 draft의 설치형·포터블 ZIP, SHA256SUMS, release-binding.json과 검증 소스를 대조한다.
- 공개 Release와 실제 다운로드, Pages에 갱신된 preview 링크를 확인한다.
- 문서에 실제 URL·실행 결과·미검증 범위를 남긴다.

## 검증과 handoff

기존 작업 폴더의 Git index는 변경하지 않고 정상적인 격리 worktree에서 필요한 문서만 반영한다. 공개 API 상태와 원 실행 증거는 프로젝트 소유 `.local-tools/publication-20261018/`에 보관한다.


## 실제 실행 결과

- Pages: https://joongi007.github.io/dlpad/; 첫 배포 run39100937703, 수동 relay가 요청한 main 배포 39103421782, 모두 success. 최초 published run39102954680의 tag 환경 거절은 별도로 보존했다.
- 릴리스: https://github.com/joongi007/dlpad/releases/tag/v0.1.0; tag v0.1.0 → source `a98afd685fb8f09ce47ab014c910be41aa39a4aa`, 원 main CI39068934195와 태그 CI39101008422 success.
- `start-pages.py`, `start-release.py`, `watch-release.py`, `verify-draft.py`, `publish-reviewed.py`, `verify-public-downloads.py`, `review-live.cjs before-release/after-release`, `verify-live-static.py`를 실제 실행했다. 모두 최종0/PASS이며 동일 실행 산출물을 재빌드하지 않았다. 저장소 homepage도 공개 Pages로 연결했다.
- 공개 전/후 headless PC·모바일4시나리오: 이미지·키보드·테마·가로 넘침·same-origin 요청·정확한 다운로드 링크 PASS; Root가 실제 캡처를 검토했다. 이번 a98 원 ZIP을 가상 Windows에 실제 설치·재검증하고 설치 앱의 Light/Dark·960/800·100/200% focus-scroll·두 번 자연 종료를 다시 검증했다. raw REVIEW_REQUIRED는 보존하고 별도 Root 검수를 기록했다. 깨끗한 실기기 인증은 주장하지 않는다.
- 조회 도우미의 첫 시도에서 미공개 draft의 공개 tag 조회404 및 artifact Accept415를 확인했다. 인증된 목록과 각 API의 올바른 형식으로 교정 후 동일 산출물 검수를 통과했다. 제품 코드·공개 파일 변경은 없다.
- 실제 HTTPS200·정적 콘텐츠5개 원본 일치. 첫 정적검사에서 .nojekyll404가 발생했고 workflow Pages의 숨김 빌드 marker를 공개 콘텐츠와 구분한 후 재검사했다. 빌드 allowlist7개 중 이 marker를 공개 파일로 주장하지 않는다.
- `python3 scripts/check_docs.py --source-only`: 1129문서 PASS; `python3 scripts/check_docs.py`: 1224문서 PASS, 원 notice133개 hash 일치(공개 기록의 최종 문구 반영 전 실행). 최종 공개 기록 기준으로 source-only1130문서 PASS 및 기본1225문서 PASS·원 notice133개 hash 일치를 추가 확인했다.

## 공개 다운로드 SHA-256

- `dlpad-0.1.0-windows-x86_64-installer.zip`: 17194332 bytes, `1a9fec772e09be2f25db02e4d12bc892f77017ec04ed33c921cd8cd49977ba94`.

- `dlpad-0.1.0-windows-x86_64-portable.zip`: 12770446 bytes, `7b86c693f74df42487a24b8030ea7b748f88622d3c08c7f1a5ef53232ae40cac`.

- `release-binding.json`: 3120 bytes, `adc83274d84b0fc321e583cbe2308f5e333c9bb4e57810fb49cd2ae487f13f5a`.

- `SHA256SUMS`: 213 bytes, `54df9075eeef3d48b704f3e5c7805579eb749a16053e64aea5b5ea1d54998f22`.

## 미검증과 다음 단계

운영 Authenticode·서명키 외부 공급·깨끗한 Windows 실기기·물리 IME/OS DPI·자동 A→B 교체·프로필 호환 및 rollback은 이번 공개로 완료되지 않는다. 앱 자동 업데이트는 미제공이며 사용자는 새 ZIP을 직접 설치한다. 첫 published Pages 배포는 tag 환경 정책에 거절됐고 main 갱신 relay로 교정했다. 실제 hosted 수동 relay와 그 토큰이 요청한 main 배포는 통과했으며, 후속 새 태그 published의 이벤트 연결은 다음 릴리스에서 확인한다. 공개 Release를 preview로 유지하고 후속 버전마다 정확한 Cargo 버전과 새 태그를 사용한다. 기존 태그/Release를 덮어쓰지 않는다. 공유 schema·Cargo/lock·사용자 미디어를 변경하지 않았다.
