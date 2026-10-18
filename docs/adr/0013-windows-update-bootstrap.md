# ADR-0013: Windows 순차 UI 실행과 업데이트 선택

- 상태: Accepted, Windows epoch1 UI 전환·복귀; 운영 배포 인수는 별도
- 날짜: 2026-10-18
- 관련: [서명 후보](0012-signed-app-update-candidates.md), [설치 감독](0009-installer-launch-supervision.md), [작업](../work-items/S11-update-activation-20261018.md)

## 실행과 권한

첫 업데이트 지원 버전은 UI와 독립된 고정 부모를 둔다. 부모가 원 실행 파일과 프로필 디렉터리 identity를 고정하고, 순서대로 하나의 UI 자식을 소유 Job에서 실행한다. 자식은 원 부모 프로세스 HANDLE, 양방향 파이프의 고정 프레임, nonce, PID·생성 시각, 전체128bit 프로필 ID를 대조한다. 후보의 읽기 전용 프로필 사전 검사와 부모 Commit 이전에는 UI·DB migration을 시작하지 않는다. 프로세스 시작과 실제 프레임·시작 복구 readiness는 다르다.

UI는 version/channel/generation 의도만 전달한다. 서명·전송·새 설치 capability와 설치 선택 기록은 부모가 소유한다. 진행률은 최신 관측 하나, worker와 결과는 각각 하나다. 준비된 세대에 대한 재시작 승인을 받은 뒤 기존 CloseWait로 모든 worker의 종료 확인을 기다린다. A의 프로세스와 원 Job0 확인 전에 B를 시작하지 않는다. B 시작 실패는 소유 Job을 종료·회수한 뒤 이전 검증된 선택 또는 원 baseline으로 한 번 복귀한다. 기존 앱·미디어·도구 파일을 덮어쓰거나 삭제하지 않는다.

새 파일 생성 HANDLE의 WRITE/DELETE 공유 예약을 실행 pin과 혼용하지 않는다. 준비 결과를 검증한 약한 readonly 원 HANDLE과 private typed evidence를 확보하고 생성 capability 전부를 닫은 뒤, 원 identity·해시와 경로를 다시 검사하며 write/delete를 막는 실행 pin으로 강화한다. 강화 실패는 실행 실패다.

## 영속 선택과 호환성

선택 기록은 현재 사용자 DPAPI, 원 journal 디렉터리·bootstrap identity, 프로필 epoch에 묶는다. create-only generation과 이전 기록 SHA를 연결하며 부분·미지·변조 namespace를 자동 삭제·수리하지 않는다. cold start에서도 서명과 현재 파일을 다시 검증한다. 이미 설치된 버전의 서명 metadata 만료는 검증 당시의 인증된 시각으로 판단하고, 새 업데이트 적용에는 현재 시각을 사용한다. 실패 sequence와 최고 sequence는 rollback으로 낮아지지 않는다.

첫 프로필 epoch1에서는 DB migration과 저장 옵션의 역방향 읽기 호환성을 유지한다. epoch 변경은 별도 호환성 설계가 필요하다. 원 baseline의 기존 crash recovery는 유지하며, 후보에만 읽기 전용 사전 검사를 요구한다. Commit 이후 후보가 쓴 사용자 데이터를 백업으로 무조건 덮어쓰지 않는다.

동일 SID의 악성 코드, 사용자가 전체 데이터를 과거 백업으로 복원한 경우의 전역 rollback 방지는 DPAPI만으로 보장하지 않는다. 하드웨어·외부 anchor가 없다. 고정 부모 자체의 보안 패치·프로토콜/epoch/키 교체는 이 v1 UI 업데이트 범위 밖이며 수동 설치 또는 별도 검증된 launcher 전환이 필요하다. 부모/자식 체인은 업데이트 횟수에 비례해 늘어나지 않는다.

## 운영키와 공급

제품은 저장소의 공개 Ed25519 키만 고정한다. 후보가 내려주는 공개키나 fixture 키를 제품 trust로 채택하지 않는다. 최초 운영키의 비공개 부분은 현재 Windows 사용자 DPAPI로 로컬 암호화 보관했고 외부로 전송하지 않았다. `scripts/sign_local_app_update.py`는 로컬 복호화 파이프와 일시적인 /tmp 0600 파일을 이용해 공개 metadata만 생성한다. [운영 안내](../UPDATE-SIGNING.md).

공개 v0.1.0에는 이 부모가 없으므로 첫 지원 버전의 수동 설치가 한 번 필요하다. 실제 Windows signed fixture의 전환·cold start·실패 복귀 7case를 근거로 이 v1 구조를 채택했다. 제품 release 빌드·설치·공개 공급과 물리 입력 인수는 별도 기록한다. 과거 v0.1.0을 자동 업데이트 지원 버전으로 표현하지 않는다.

## 최초 공개 구현 검증 (2026-10-19)

최초 지원 공개 버전은 v0.1.2/source9ed25f1297b59bfc922e0659279a56a227614302다. 전체Windows CI·같은실행패키지·실제설치/앱실행을검증했고, 운영키의공식공급으로 review A0.1.1의고정부모아래 B0.1.2 UI전환 및동일프로필cold선택을실제로확인했다. 원UI종료후새UI생성·같은Job/직계계보/생성시각/정확imageSHA/프로필경계·nonce·자연exit0를대조했다. 독립DPAPI해독이나A부모자체갱신을검증한것은아니다. [원본과 인수 범위](../work-items/S11-update-activation-20261018.md).
