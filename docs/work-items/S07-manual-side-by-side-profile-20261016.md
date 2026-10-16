# S07: 두 개발 설치의 수동 전환과 합성 프로필 호환 시험

- 상태: done-scoped — 두 개발 snapshot의 수동 전환·프로필 보존·B 충돌 후 A 재실행 검증
- 우선순위: P0
- 요구사항: R08, R11 / ROADMAP S07 복구·업그레이드 시험
- 담당: Root 통합·판정, field_reveal 새 native helper, search_reveal 읽기 전용 DB/원 A 검토
- 수정 파일: 새 .local-tools/session-recovery-20261013/s07-manual-side-by-side-profile-native-v1 및 v2 후보와 이 작업 기록. 기존 Live/S/schema/Cargo/lock 변경 없음.
- 의존성: 실제 whole31 원 Root검토·보존된 default A PE/payload, current whole32/Root검토/default3PE B, 현재 production API fixture.

## 범위 결정과 근거

원 REQUIREMENTS R11·ROADMAP S07·Windows 범위 계약의 실제 설치/복구/업그레이드 시험은 유지한다. ADR0005는 Proposed이며 전체 durable revision/cold writer/자동 launcher/일반 DB downgrade 구현을 사용자 확정 최초 P0 의무로 상향하지 않는다. Accepted ADR0009의 검증 후 명시 감독 실행을 따른다. 원문15파일의 행·SHA와 해석은 `.local-tools/session-recovery-20261013/s07-p0-scope-audit-v1/REPORT.md`에 기록했다. 현재 지원하는 새 폴더 설치 정책으로 서로 다른 개발 snapshot의 실제 수동 전환과 프로필 보존을 시험한다. 자동 앱 updater는 사용자 지정 기능·Git/CI 이후 후속 순서를 유지한다.

## 인수 조건

원 default v31 A와 current default v32 B의 서로 다른 실제 source/PE/payload·별도 새 bundle/journal을 고정한다. 같은0.1.0 개발 빌드 간 시험으로 한정한다. 진단 feature PE나 같은 exe의 두 복사본을 버전 전환 근거로 쓰지 않는다. 사용자 Documents·실제 영상 대신 새 소유 TEMP의 합성 appdata/media/history/settings를 사용한다.

한 프로필을 공유하는 A→B→A를 순차 실행하며 매 앱·supervisor 자연0/Job0 뒤에만 다음 실행을 허용한다. 실제 설정·이력·라이브러리 읽기와 작은 명시적 설정 쓰기/B→A 읽기 및 원 데이터 보존을 확인한다. 실제 DB나 파일을 임의 복원해 성공을 만들지 않는다. 새 B의 실패/취소/partial/불일치 경계에서 원 A·기록·프로필 보존과 A 실행 가능성을 확인한다. cold 별도 과정으로 현재 설치를 다시 검증한다. GUI는 private desktop만 사용하며 global input/clipboard/OS viewer/visible fallback은 없다.

## 한계와 검증

이 작업은 자동 upgrade·자동 rollback·기존 active pointer 교체·일반 미래 DB downgrade·전원 손실 commit·clean OS/서명/공개 배포를 제공하거나 검증하지 않는다. 호환성 실패가 실제 관측되면 원인에 맞는 최소 안전 수정이 필요하며 후속 updater라는 이유로 생략하지 않는다.

기존 journal v4의 실제 Windows17/0/0·strict engine clippy0은 별도 기반 실험의 결과로 보존한다. 새 실행/프로필 전환 시험의 성공으로 소급하지 않는다. 실제 명령·결과·원본 SHA/nativeID·독립 Root 검토는 실행 후 기록한다.

## 실제 실행과 독립 검토

Root는 `build_fixture.py`의 offline lock 생성/locked release 빌드를 실제0으로 완료했다. `v2/collect.py`로25 operations를 실행했고, 예상 B 충돌 한 개만 exact exit1/AlreadyExists이며 다른24개는0이다. A31→B32→A31 각 앱·CLI가 자연0/Job0·private desktop 종료·입력 desktop 불변으로 끝났다. 같은 합성 Queued3/파일2와 설정·원 옵션·저장 루트·sentinel이 보존됐고 Light→Dark→Dark를 확인했다. B 기존 설치 root의 충돌은 mutation/cap acquisition false, 전후55 snapshot entries가 같았으며 그 뒤 A가 실제로 다시 실행됐다.

Root는66pins·원25로그·3 native receipts·현재675 source entries를 재대조했다. 원 jobs/library DB의 bytes/nativeID도 이번에는 모두 같았다. Root가 별도 immutable readonly SQLite 조회로 Queued3·attempts0·artifacts0를 확인했고 DB SHA가 유지됐다. 원 PNG10개 중 Root3개/독립 검토자10개를 직접 봤다. [Root 검토](../validation/evidence/s07-manual-profile-private-v2-20261016/root-review.json).

동일 v0.1.0이지만 서로 다른 PE/source의 개발 snapshot 간 좁은 호환 시험이다. 완료·실패 이력/미래 schema downgrade·자동 교체/rollback·crash/powerloss는 이 시험 범위 밖이다. S07의 지원하는 수동 설치 정책에 대한 실제 전환·실패 보존 근거이며, 서명·clean OS·전체 P0 완료를 뜻하지 않는다.
