# S11: 작은 창의 업데이트 설명 스크롤 정리

- 상태: proposed
- 분류: ux
- 우선순위: P1
- 요구사항: R08, R10
- 담당/수정 범위: 후속 담당; crates/app/src/accessible_app_update.rs, 필요 시 중앙 catalog; 이 작업은 기록만 작성
- 선행 작업: Windows v0.1.2 설치 앱의 private GUI 검수

## 사용자 결과

작은 창과 200% 배율에서 릴리스 설명을 읽을 때 내부 설명과 설정 본문의 이중 스크롤을 줄인다.

## 계약과 범위

최신 설치된 CPU의 Light/Dark 및 재실행 검수에서 관측했다. 현재 설명은 기본 접힘이며 주요 버튼 겹침은 없으므로 이번 공개의 차단 결함으로 분류하지 않았다. 설명을 펼친 상태의 높이·스크롤 소유권과 Markdown 표기를 함께 검토한다. 다운로드/서명/부모 프로세스 계약은 변경하지 않는다.

## 인수 조건

800/960 폭과 200% 앱 배율, Light/Dark, 긴 설명·빈 설명에서 다음 행동이 가려지지 않고 설명 끝까지 읽을 수 있어야 한다. 가상 화면에서 실제 PNG와 키보드 포커스를 확인한다.

## 검증과 handoff

아직 구현하지 않았다. Root 검수 원본은 ignored `.local-tools/update-publication-20261018/private-gui/hosted-installed-v2/root-review.json` 및 원 PNG다. 물리 IME·OS DPI·스크린리더 인수를 완료로 표현하지 않는다.
