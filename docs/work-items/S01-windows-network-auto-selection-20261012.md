# Windows 네트워크 관측의 자동 선택

- 상태: in-progress
- 분류: UX
- 우선순위: P0
- 요구사항: R01, R08
- 담당/수정 범위: root — 공유 engine/app/Cargo 통합; windows_auto — /tmp 후보만
- 선행 작업: 현재 UI/network 통합 및 사용자의 기본값·고급 옵션 피드백

## 사용자 결과

사용자가 이름을 입력하지 않아도 유효한 활성 인터페이스를 관측하고, 사용자 선택·끄기·경로 변경·baseline 안전성을 보존한다.

## 계약과 범위

Windows 제품에 집중한다. 하위 담당은 프로젝트 읽기 및 /tmp 후보 준비만 수행하며 native/Cargo/공유 source를 변경하지 않는다. root가 인터페이스·schema·Cargo와 통합 검증을 조정한다.

## 인수 조건

사용자가 이름을 입력하지 않아도 유효한 활성 인터페이스를 관측하고, 사용자 선택·끄기·경로 변경·baseline 안전성을 보존한다. 실제 Windows 및 core 실패 경로를 별도로 확인한다.

## 검증과 handoff

독립 후보 준비 단계이며 제품 구현 완료로 표시하지 않는다.

## Root 통합 결정

기본 route 관측에서 IPv4 후보를 먼저, 없으면 IPv6를 사용한다. 동일 family는 route+interface metric 합과 LUID 순서로 선택한다. 실제 다운로드 목적지의 연결과 다를 수 있으므로 다운로드 연결로 표시하지 않는다. 기존 단일 worker·counter epoch를 유지하고 windows-sys에 WinSock feature만 추가했다. lock/version은 보존한다. source 통합 단계이며 실제 Windows 결과는 아직 미검증이다.

실행 결과: 첫 Windows targeted는 windows-sys route의 BOOLEAN이 Rust bool이라는 binding 차이로 컴파일 exit101. 원 실패 로그를 보존하고 Loopback/Connected/DisableDefaultRoutes를 bool 조건으로 교정했다. 다음 실행 결과 전 성공으로 표시하지 않는다.

Windows targeted v2는 exit0, 86 성공/0 실패/1 ignored. Root가 원 로그와 source map, actual counter/worker 회귀, natural Job0 및 입력 desktop 보존을 직접 대조했다. 실제 기본 자동 연결·rate·선택 메뉴 픽셀 검수와 전체 검사는 남았다.

전체 검사 첫 실행은 fmt 통과 후 strict clippy가 route cache tuple의 복잡도를 거절했다. 원 실패를 보존하고 의미를 유지하는 CachedInterface 타입 이름으로 수정했다. 경고 억제를 추가하지 않았다.

제품 실제 Windows healthy private GUI v4 exit0. 첫 자동 이더넷 수신/송신 관측, 목록, 끄기 별도 상태·rate 제거, 없는 수동 이름 유지, 자동 복귀 및 원 combo runtime ID 유지를 확인했다. Root가960x640/System100% actual auto/popup PNG를 직접 검토했다. vendor가 UIA LabeledBy를 게시하지 않아 원Name/Value/type으로 식별했으며 원 실패는 보존했다. 실제 route 변경·IPv6/VPN·물리 입력/reader 및 선택 저장은 이 scope가 아니다. whole1227/0/30 및 lock 불변을 대조했다.
