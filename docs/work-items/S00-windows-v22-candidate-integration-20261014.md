# S00: Windows v22 후보 통합과 최신 검증

- 상태: in-progress
- 분류: integration
- 우선순위: P0
- 요구사항: R01, R02, R06, R08, R11, R13
- 담당/수정 범위: root — 공유 Cargo/schema와 제품 통합·최종 검증; advanced_units — 독립 드래그 시험 fixture; installer_batch — 분리 설치 helper와 원자료 검토; windows_auto — 독립 UI/CI·비용 수집 후보
- 선행 작업: v21 whole/package, 격리 후보 앱/Core/engine 검증, 별도 Windows 드래그 정책7시험

## 사용자 결과

다운로드·파일 목록·작업 이력·설정의 네 개 상위 탭을 제공한다. 기본 저장 폴더는 설정에서 관리하며 파일 목록은 원 파일 미리보기·열기·새 이름 복사를 제공한다. 제목 파일명 옵션을 기본 제공한다. 화면을 바꿔도 복사 진행·중단 대기를 보여주고 실패 정보 위치를 정확히 안내한다. 초기 확대·테마 화면의 배경을 채우며 Windows 드래그 등록 실패 때문에 창 생성이 중단되지 않도록 한다.

## 계약과 범위

Root가 동결된 `tool-isolated-source`와 v21 원파일을 해시로 대조하여 변경 파일만 보존·적용한다. 이전 ZIP·실패 원로그는 변경하지 않는다. 공유 옵션·재개·publication 계약은 ADR0010 후보에 따른다. Windows typed Shell broker는 외부 player를 installer Job에 넣지 않는 후보이며 실제 사용자 기본 앱 수명은 별도 인수다. 도구 cache는 표시 정보이고 실행 권한은 재검증 결과에서만 받는다.

Cargo의 windows0.61.3 직접 dependency edge는 기존 잠금 버전·체크섬을 재사용하며 새 다운로드 패키지 버전은 추가하지 않는다. 드래그 패치는 기존 SDK0.52를 사용하고 새 dependency/feature/worker를 만들지 않는다. 크기·CPU·RSS 변화는 최신 빌드·비용 측정 뒤 기록한다. 전체 macOS/Linux 검증과 공개 배포·서명·외부 계정·호스트 CI 업로드는 이 작업의 성공으로 대체하지 않는다.

## 인수 조건

Windows 현재 소스에서 fmt·strict all-target clippy·workspace test·release 네 명령을 실제 실행하고 원로그·소스 sandwich·실행 파일3개를 Root가 독립 대조한다. 새 package graph·notice·ZIP SHA/CRC를 actual whole과 연결한다. private desktop에서 최신73화면을 직접 검토하고 정상 설치·기존 경로 거절·소유1바이트 변조 거부·긴 한글 경로 앱 HWND와 자연 종료를 검사한다. 원 파일·journal·Job0/private desktop 종료를 유지한다. 로컬 CI/비용과 실제 드래그 메시지 fixture는 최신 소스에 연결하며 사용자 물리 검수는 별도 남긴다.

## 검증과 handoff

격리 후보의 앱425/Core37/engine566, 드래그 정책·스타일7시험은 실제 Windows에서 통과했다. root-review는 각각 검증 범위와 원로그 해시를 보존한다. 최신 footer 회귀는 화면 간 실제Context2개와 export 필터, strict 검사를 통과했고 Root 대조 중이다. 최초 footer-v6은 formatter 완료 전 phase selector를 읽어 KeyError로 준비 단계에서 실패했다. formatter 완료 뒤 새 v7 결과로 구분한다. 테스트 성공으로 덮지 않는다.

전체 최신 빌드·최신 설치 ZIP·실제 GUI 및 비용은 아직 완료하지 않았다. Git 상태 조회는 exit128이고 기존 `.git`을 그대로 보존했다. 다음 작업은 동결 후보 채택→새 whole→package graph/ZIP→native 설치·73화면→최신 CI/비용이다.

Root 후보 채택 완료: 변경53파일(신규5 포함), 현재 source669. 원 v21 전체 source map을 대조하고 변경 전 파일과 before/afterSHA를 `root-v22-source-adoption-v1/receipt.json`에 보존했다. 현재 v22 전체 fmt·clippy·test·release를 fresh evidence로 실제 실행 중이다. v21 ZIP을 변경하지 않았다. footer-v7 실제2+12회귀와 strict 검사 및 source sandwich를 Root가 독립 대조했다.


2026-10-14 최신 결과: source669의 네 명령을 실제 Windows에서 완료하고 Root raw/source/current3를 대조했다(1,524 PASS/0 FAIL/42 ignored,43 harness). CPU10,234,880B는v21 대비122,880B 증가, installer4,425,728B는512B 감소, CLI978,432B 크기 동일이며SHA는 새whole에 기록한다. 별도 실제 드래그 메시지4case는 원WndProc/HDROPinvalid/동일WindowIdDestroyed/IsWindow0·naturalJob0를 확인했다. 원 fixture의 즉시IsWindow검사 실패는 보존했다.

v22 portable18,200,631B/installer17,150,135B ZIP을 생성하고 Root1,129입력·474멤버 SHA/CRC·456원notice를 독립 대조했다. selected258 missing0, whole-lock43미확인/서명·legal·공개릴리스 미완료를 유지한다. 정상 설치 자연0/Job0·44PNG직접 팀 검토와577/598긴경로 창·자연0·원20객체6부모/5payload/5journal을 독립 대조했다. 처음 긴 경로 collector의 일반 경로 가정 실패는 구분했다.

로컬 CI는 새로운 소유 export2,010 records에서 실제 MSVC host15단계·8Cargo·7capture·43harness/1,524/0/42를 독립 대조했다. 원 프로젝트.git은 변경하지 않았다. 최신73화면은 새4탭 스크롤 경계·legacy.NET30159 registry·오래된FFmpeg caption 수집 오류를 교정한 뒤 시간 상한에 도달했다. 초기·Dark선택·Dark재시작을 각각180worker/230host로 나눈 명시적3scope합73contract로 실행 중이며 전체화면 성공으로 승격하지 않는다. 18구간 비용은 동시nativeprobe 종료 뒤 실행한다.

현재 v22 변조 거부를 Root가 원로그·현재33pins·generated7·소유변조exe SHA/크기/첫byte·다른4파일·journal5개로 독립 대조했다. installer 자연1, 원Job 총프로세스1 유지와 앱 자식 생성 없음, 검증가능/실행불가/기존대상설치불가를 확인했다. [독립 결과](../validation/evidence/windows-msvc-ui-feedback-v22-20261014-command-negative/root-review.json). 초기73화면 시험의 설정DB 존재 가정 오류는26화면/앱자연0와 분리하여 FAIL을 보존한다. 최신18구간 비용은 동시native실행없이 진행 중이다.

최신 비용 실제 실행은 strict budget FAIL이다. 세60초 평균 CPU0.598640/0.884629/0.650723%, 메모리18구간PASS이나 CPU18중16PASS/2FAIL이다. 두FAIL은round2window3/5 각각1.093468/1.092684%, 동일1,093,750×100ns다. 해당 thread attribution은thread set churn으로불완전하므로 부분thread표에서전체원인을단정하지않는다. 새측정/성공으로덮지않고 원raw·독립검토·성능진단을후속으로유지한다. [원측정](../validation/evidence/current-ui-feedback-v22-default18-cost-20261014/collector-result.json).

최신73화면 실제검사 및 Root 독립검토 완료: compile+세native scope 자연0/원Job0, current109pins/73PNG·전후source669, 설정초기legacyLight7B/DB미생성→실제Dark저장→같은원루트재시작Dark/읽기전용3SQLite를 대조했다. Root12+child61개 원PNG를 직접검수하고 전체73해시·크기·해상도를 다시 확인했다.150/200설정 상단일부offviewport와URL즉시/readback프레임차이는pixel만으로 원인을 판정하지 않는다. 물리hover·IME·reader·picker·전체GUI인수는별도다. [원로그 검토](../validation/evidence/current-ui-feedback-v22-app73-three-scope-v2-20261014/root-review.json), [73화면 검토](../validation/evidence/current-ui-feedback-v22-app73-three-scope-v2-20261014/root-team-pixel-review.json). Root 최초감사도구가 native JSON과추가readonly-review필드를한번에비교하여setupFAIL했으며, 추가필드하나만명시적으로분리하고원필드전체를대조했다. 원감사스크립트는보존했다.

README의이전checkpoint는docs/validation의허용된Markdown경로로옮기고상대링크를이동위치에맞게조정했다. 이전Root위치에서생성했던임시역사파일은원byteSHA6b4f37144a69f61e4c10b0a4a9406bf72f4ac6ab24583682e1aea2a6651d50ab와백업을확인한뒤정리했다. 기존source-export allowlist를변경하지않고후속export에서README기록링크도포함되게한다.
