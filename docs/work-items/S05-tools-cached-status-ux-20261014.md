# S05: 도구 캐시 상태와 중복 설치 UX

- 상태: in-progress
- 분류: bug
- 우선순위: P0
- 요구사항: R05, R08
- 담당/수정 범위: installer_batch accessible_tools.rs; tool_update_ui.rs 직접 관련 embedded 표시 변경은 Root 합의 후; 본 문서/B helper만
- 선행 작업: 현재 3slot cache/한 worker·sync1/세대·CloseWait 계약, Root catalog/Main 통합

## 사용자 결과

도구 탭 재진입은 기존 결과를 즉시 표시하며 갱신 중에도 값을 유지한다. 상단에서 설치기록/확인중/실패·설치 버전·현재사용버전을 구분한다. 동일 current pinned 관리 설치기록이 있으면 중복 설치를 막고 새버전 확인·보관버전 사용을 안내한다.

## 계약과 범위

README→PRODUCT→CONTEXT→R05/R08→ADR0008·DESIGN·INTEGRATIONS를 읽었다. UIthread 파일/DB/네트워크/도구프로세스 금지. 기존 cache는 metadata 표시이며 실행/제거권한이 아니다. 행동마다 기존worker의 실제 파일/runtime검증을 유지한다. 새SQLite/schema/상주worker/Cargo/Main/catalog직접수정 제외. Root와 다른agent의파일소유 분리를 지킨다.

## 인수 조건

actualContext가 설치상태/기록버전/current사용버전과 업데이트중old값유지·실패값유지를 실제그린다. 동일pin있음/다른version·target·hash·External 구별과 primary설치disabled/명시갱신별도 행동 확인. 작업중/Stop/닫기/세대오래된ACK·NoRoot gate 보존. 실제Windows UI/native 검증은 Root가수행하며 Context통과만으로 물리승인하지 않는다.

## 검증과 handoff

기존actual cached tab/failedrefresh/latestslot/rowstable/generation 시험을 발견했다. 현재상태가 공급설명·업데이트pane 아래에 있고 Install은이미있는pin에도항상활성인 결함을 확인했다. 상단표시/중복설치gate를 좁게수정한다. 원본backup/pins는 B/tools-cached-status-ux-v1에 기록했다. 새native/Cargo실행은 아직0이다.

### 구현 및 검증 준비

Root와cached탭복귀noI/O정책·catalog6·embedded중복현재row제거범위를합의하고소유2파일을실제수정했다. 캐시가있는탭은Stale표시와함께즉시보여주고첫inventoryNone만자동검사한다. 명시Refresh/Install/Use/Select/Remove fresh worker와generation/Stop/CloseWait유지. 동일Managed fixedmanifest record이면Install버튼및dispatch거절, 최신조회는별도유지. status위계와기록버전/currentsource active버전을상단에둔다.

새actualContext2시험과기존cached transition3회귀를준비했다. file-onlyrustfmt1.94 exit0이며 Cargo/Windows시험/nativeGUI실행은아직0이다. B/tools-cached-status-ux-v1 source-pins/HANDOFF/patch에원본·변경·원인·비용·Root실행필터를기록했다. Context통과·속도실측·GUI인수는Root후속검증전미검증으로유지한다.
