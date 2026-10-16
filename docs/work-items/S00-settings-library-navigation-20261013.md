# S00: 설정 중심 탐색과 파일 목록 역할 정리

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R02, R08, R11
- 담당/수정 범위: root — accessible.rs, accessible_library.rs, accessible_metrics.rs, catalog.rs
- 선행 작업: v20 사용자 검수; range/installer 후보와 파일 분리

## 사용자 결과

상위 탐색은 다운로드, 파일 목록, 작업 이력, 설정 순서로 정리한다. 폴더 변경과 도구 관리는 설정에 모으고 파일 목록은 파일 이름/정보/미리보기/열기에 집중한다. 저장 폴더의 여유 공간은 측정 대상 경로가 드러나도록 표시한다.

## 계약과 범위

기존 library worker/설정 저장/미리보기 및 process pin을 재사용한다. 물리 볼륨 이름을 경로 문자로 추정해 확정하지 않는다. 현재 space API는 저장 폴더에서 호출한 GetDiskFreeSpaceExW의 사용자 사용 가능 bytes이며 총 용량이 아니다. root path revision/표시 stale gate와 I/O worker 계약을 유지한다. 새 dependency/schema 변경은 없다.

## 인수 조건

4개 상위 목적이 중복 없이 탐색된다. 폴더 변경은 기존 검증/worker 결과로만 적용한다. 목록 미리보기는 기존 범위/삭제 gate와 busy/CloseWait를 유지한다. 여유 용량 tooltip에 실제 측정 경로와 값의 의미가 나타난다. 합성 임시 media/가상 화면에서 검증한다.

## 검증과 handoff

현재 경로는 self.output_path() → metrics.set_root → worker.volume_free_bytes이다. 사용자에게 나온594.1GiB의 실제 당시 경로는 아직 확인하지 않았으며 C:의 값으로 보고하지 않는다. 기존 v20 설치본은 source 변경과 별도로 보존한다.

2026-10-14: 격리 후보에 상위4탭과 설정 일반/도구/저장 위치를 통합했다. 첫 Windows 검사는 shared Cargo dependency edge 미반영으로 --locked 실패했고, lock에 기존 windows0.61.3 root edge만 추가했다. 다음 검사는 Candidate initializer의 settings_section 누락으로 컴파일 실패했다. 이를 General로 초기화한 feedback-settings-broker-v3를 실행 중이다. 두 실패는 기능 시험 PASS로 세지 않는다. 저장 위치 화면은 현재 다운로드 draft 편집이며 영구 기본값 저장과 파일 목록 폴더 편집 이동은 아직 미완료다.

추가 통합: 설정에서기본폴더 원Apply/Default/네이티브picker를재사용하고목록은현재root와Refresh/Next/파일작업만그린다. accepted locationserial은started/ACK한번소비하며Root의현재output+output_original baseline과CloseWait를비교한다. 위치준비/영구설정worker와schema는변경하지않는다. 이번요청만의경로는별도접힌옵션으로구분한다. Windows compile에Root visibility와AccessKit&String타입오류가발견되어수정했다. 이후새 locationContext2+serial1+RootstaleACK1은실제통과했고Library전체는44PASS2FAIL: 이동전ChooseFolder노출/초기emptyhint미노출기대가새목록역할과충돌했다. 새context는목록에폴더편집이없음을검사하며이전fixture두개의 UI기대도Refresh/Next와바로보이는emptyhint로수정한다. 최초FAIL은보존한다. startup복구Apply와사용자intent구분은독립검토중이며제품소스인수와달리후보검사만기록한다.

추가 Root 재현: feedback-location-startup-red-v1에서 actual Recovery branch가 생성한 Apply가사용자 outputintent를mint하는오류를0PASS1FAIL로확인했다. startup전용apply_observed_root는원request/serial/commit/ACK를유지하면서새userstarted만발행하지않는다. 원typed수동입력/늦은ACK/CloseWaitguard는유지한다. feedback-location-copy-final-green-v4의startup/보조회귀각1은통과했고copyclosewait1도통과했지만matrix는busyRefresh활성UI를발견했다. backend는queue1로안전했으나UI는pending중새로고침을disable하여Next/파일행동과일관되게했고원watch/refreshbackend를변경하지않았다. green-v5진행중이다. 수정전FAIL과compiler/misplacedtestscope실패는보존하며현재전체성공으로쓰지않는다.
