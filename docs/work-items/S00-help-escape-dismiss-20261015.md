# S00: Escape 뒤 자동 설명 재표시 억제

- 상태: in-progress
- 분류: bug
- 우선순위: P0
- 요구사항: R07, R10
- 담당/수정 범위: advanced_units의 ui_components.rs scratch/help만; 통합·실행은 Root
- 선행 작업: whole27 및 실제 Help-v2 Escape 원PNG

## 사용자 결과

Escape로 고정 설명을 닫으면 owner focus를 유지하며 설명 본문도 사라집니다. 같은 focus/hover를 유지하면 다시 뜨지 않고, 새 클릭·focus 재진입·pointer hover 재진입에 다시 열 수 있습니다.

## 계약과 범위

원 Help-v2의 help-after-escape.png에 제목 없는 본문 overlay가 남았습니다. source는 popup을 닫고 owner focus를 복구한 직후 context_help를 다시 호출합니다. 위젯별 고정 크기 temp record로 이 Help의 자동 tooltip만 억제합니다. 기존 ID/Response/접근성/아이콘/popup/밖 클릭/disabled 정책과 다른 context_help는 유지합니다. 영구 저장/schema/timer/I/O/새 의존성은 없습니다.

## 인수 조건

실제 Context에서 Escape 다음 3 settled frame은 body/title 0, popup 닫힘, 원 owner focus 유지입니다. 연속 focus/hover는 재표시하지 않고 pointer exit/reenter·Tab away/Shift+Tab back·새 키보드/mouse 활성화는 다시 설명을 보여야 합니다. 실제 다른 Response 밖 클릭은 새 target focus를 유지합니다. 기존 Help 3개 회귀를 보존합니다.

## 검증과 handoff

Linux rustfmt2024 두 후보 파일이 성공했습니다. Cargo/native는 미실행이며 제품·Live·S·evidence·문서는 변경하지 않았습니다. 원본/RED/GREEN/patch/pins는 B/help-escape-dismiss-v1에 보존합니다. 임시 payload는 두 bool과 frame 한 개로 고정 크기이며 기존 고정 field/tool-slot ID만 사용합니다. 반복 프레임에는 기존 entry를 갱신해 새 Box 할당을 피합니다. Root RED/GREEN 및 후속 native pixel 검증을 기다립니다. CPU 예산 해결·물리 입력·reader·전체 UI 완료를 주장하지 않습니다.

Root: 검색 GREEN 실행과 독립적으로 후속 RED/GREEN 통합을 준비 중입니다. 현재 실행 파일은 whole27 그대로입니다.

Root 실제 RED compile 성공 0 PASS/1 FAIL로 Escape 후 본문 남음을 재현했습니다. GREEN-v1 actual Help 4회귀는 1 PASS/3 FAIL(신규 및 기존 아이콘/키보드 회귀)로 미채택입니다. source/S 스냅샷 unchanged. 기존 제품은 whole27 그대로 보존하고 새 scratch 수정 후보를 병렬 준비 중입니다.

후속 GREEN-v2 실제 Windows Context는 3 PASS/1 FAIL입니다. Context 데이터 잠금 안의 Response.clicked/gained_focus 재진입을 밖으로 옮겨 기존 3회귀의 deadlock을 해결했습니다. 남은 실패는 새 Shift+Tab 시험의 즉시 포커스 가정이며 vendor memory의 id_next_frame 계약상 한 프레임 지연됩니다. v3는 RED/GREEN 둘 다 그 다음 실제 프레임을 관측하며 기존 본문/포커스 조건은 유지합니다. 아직 채택하지 않았습니다.

직전 turn 검증 정리: check_docs는 중간 실행에서1123 PASS/114 upstream notices를 확인했습니다. 후속 실행 핸들92662는 이번 복구 시 Unknown process id로 관측 불가였으므로 그 실행을 성공으로 기록하지 않습니다. 새 문서 수정 후 다시 검사합니다.

2026-10-15 후속 실제 검수에서 200% 환경yt-dlp Help의Escape후4개100ms표본은title/body없음/원ownerfocus유지였으나다섯째에bodytooltip이재표시됐다. 원E/current-protected-help-private-v2-20261015와실패PNG를보존한다. field_reveal이B/help-dismiss-reentry-v1에서window/logicalfocus·Escape반환gained_focus·popup해제hover를실제Context로분리재현하며Root가공유ui_components.rs채택·통합검증을담당한다. 실측분리전어느가설도원인확정으로쓰지않는다.
