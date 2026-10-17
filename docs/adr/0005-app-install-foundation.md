# ADR 0005: 앱 설치 기반과 도구 설치 권한의 분리

- 상태: Proposed
- 관련 요구사항: R11
- 작업: [Windows 설치 기반](../work-items/S07-windows-app-install-foundation.md)

## 문맥

개발 ZIP 생성은 사용자별 앱 설치 수명과 동일하지 않다. 관리 외부 도구의 registry·root lock·삭제 권한을 앱 실행 파일 설치에 그대로 적용하지 않는다. 사용자 updater 지시는 기능과 Git/CI 이후의 후속 목표다.

## 제안과 구현 경계

Bundle manifest는 bounded JSON의 파일명·크기·SHA256·대상 설명이며 publisher trust가 아니다. Payload observation은 원 열린 파일의 bounded streaming hash·PE64 구조·native identity 전후 관측이며 이후 내용의 불변성이나 실행/삭제 권한을 부여하지 않는다. Windows에서는 원 HANDLE의 ReOpenFile로 별도 읽기 handle을 얻어 caller 커서를 보존하고, Unix에서는 positional read로 공유 커서를 움직이지 않는다. ReOpenFile 실패를 pathname 재open으로 복구하지 않는다. [원 handle 재열기 계약](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-reopenfile). FreshRoot는 새 Windows 디렉터리의 원 생성 HANDLE·현재 SID의 private ACL·조상과 root identity를 보유하는 capability다. 기존 경로를 채택하거나 지우지 않는다.

CreateDirectoryW 후 path를 다시 여는 방식은 사이에 leaf가 교체되면 대체물을 최초 원 identity로 채택할 수 있다. 원 parent HANDLE에 상대적인 NtCreateFile(FILE_CREATE)로 생성과 원 HANDLE 획득을 함께 수행한다. SHARE_DELETE pin은 namespace 잠금이 아니므로 후속 작업 전후에 원 path/identity/ACL을 재검증한다. 생성 뒤 검증 실패의 새 폴더는 추측 경로로 삭제하지 않고 보존한다. [Microsoft 호출 계약](https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntcreatefile).

설치 publication·durable journal·실제 재시작·정확 제거는 후속 coordinator의 별도 상태/인수다. 현재는 active version pointer·upgrade·DB backup/migration·registry/PATH·서명·공개 배포·updater를 제공하지 않는다. 합성 TEMP 인수는 실제 clean-machine 설치를 대체하지 않는다.

## 비용과 검증

기존 sha2/serde/windows-sys를 사용한다. windows-sys의 Wdk_Foundation/Wdk_Storage_FileSystem 선언 feature 추가는 새 crate나 background process를 추가하지 않는다. Manifest 최대64KiB·64개 항목, payload chunk/header 각각64KiB·총256MiB 상한이다. 동시 실제 payload 관측은 호출자가 제한하고 긴 I/O를 UI thread에서 수행하지 않는다. 최초 native fmt PASS/Clippy FAIL을 보존했으며 수정 후 실제 전체 검증 전에는 채택 완료로 표시하지 않는다.

후속 기반 검증: source548 Windows 전체4명령과신규15actual시험, 실제CPU/notice의 boundedPE/hash/nativeID/cursor17→17·한글freshroot/privateACL/sentinel 보존이 통과했다. 기본CPU4f7a…는7,917,568B로이전대비크기증가0이다. 최초Clippy2실패는보존했다. [root실측](../validation/evidence/windows-app-install-foundation-real-payload-20261006-continue/root-review.json).

현재 후속 제안은 PreparedBundle이다. 새 root-relative 파일에 검증된 source를 복사·sync한 후 정확 entry nativeID/size/SHA를 PreparedRecord로 기록한다. 이 상태는 active installation·signed trust가 아니다. PrepareFailure는 원 partial handle과 target hint를 보존하며 hint는 삭제 권한이 아니다. Ready marker는 CreateNew이며 기존 설치·active pointer·재부팅 회수·updater·제거를 제공하지 않는다. [후속 계약](../work-items/S07-windows-app-prepare-transaction.md).

2026-10-06 준비 트랜잭션 인수: source550 Windows 전체4검사·app_install 22 실제 tests·CPU copy/정확 namespace/marker·sentinel·copied 원 exe의 두 private 실행/Light DB persistence PASS. 설치 등록/upgrade/uninstall/trust 완료가 아니다. 다음 explicit readonly inspection은 PreparedInspection의 별도 cap를 사용하며 FreshRoot를 기존 경로 채택용으로 변경하지 않는다. 이 후속은 통합 검증 전이며 ADR 전체 Proposed를 유지한다.

후속 fresh discard 구현은 새 원 생성 HANDLE에만 DELETE를 요청하고 원 HANDLE FileDispositionInfo를 사용한다. namespace/ACL/ID/streaming SHA를 각 단계 전후 검증하며 failure remainder로 원 미처리 capability를 보존한다. 원 객체의 SHARE_DELETE는 경로 잠금이 아니고 unknown 삽입은 실패로 보존한다. 재개방에 DELETE 권한 부여·recursive cleanup·cold uninstall·secure erasure를 제공하지 않는다. 신규 dependency/lock/schema 변경 없음; 실제 통합 검증 전 단계다.

Fresh discard source555 인수는 전체4검사·40회귀·actual4bundles 및 원prepare/CLI 제어호환을 통과했다. 다음 개발용 등록 결합은 원 record/seal 두 CREATE_NEW HANDLE IDs를 먼저 얻어 자기 digest 없는 record canonical bytes→domain SHA→seal 순서로 기록한다. nonce는 상호결합값이며 같은SID의 공격 방어비밀이 아니다. fresh registration이 preserve하는 원caps와 향후 readonly cold inspection/explicit removal authority를 분리한다. 등록은 현재 구현 중이며 ADR Proposed를 유지한다. [등록 계약](../work-items/S07-windows-fresh-app-registration.md).

source558 fresh 등록은 Windows 전체4검사·49회귀 및 실제CPU 원ready/record/seal IDs·SHA 결합/외부보존·별도native6ID대조를 통과했다. 후속 RegisteredInspection은 기존 ReadRoot/ReadFile만 이용하는 readonly consistency이며 삭제권한을부여하지 않는다. 제안전체는cold삭제·active설치·cleanOS인수를 마치지 않아 Proposed로유지한다.

source560 RegisteredInspection 전체4검사·53회귀/별도originalregisteredCPU 재검증·4typedfailure/caller37 보존인수PASS. 원bytes/canonicalseal/nativeIDs consistency readonlycap이며cold삭제권한과역사적소유권증명이아니다. 다음개발CLI명시등록/readonly명령은정식activeinstaller완료와구분한다.


2026-10-06 개발 CLI 네 명령 prepare/inspect/prepare-register/inspect-registered 연결을 source560 전체4검사·설치53/parser3회귀·실제16명령과 별도원nativeIDs·현재legacy제어로 검증했다. 등록 inspection은readonly이고 active설치/삭제권한/서명trust로승격하지않는다. 다음 durable 외부authority anchor와 cold removal은 후속이며 ADR 전체 Proposed 상태를 유지한다.

외부receipt는총64KiB와중첩tuple64KiB-2048 admission을사용해생성전보수거절하고실제2nativeIDs의canonical전체상한을다시검사한다. 생성실패opaquecaps는보존하고Drop삭제0. source563 전체4검사/61회귀/실제receipt2305B·5case/native2IDs 통과. 원파일사본은서명trust/역사적소유권/자동cold삭제authority가아니다. 신규dependency/lock변경없고GUI7917568B 불변; 기존GUI비용소급0.

ReceiptInspection은원readonly root/filecap에서외부receipt의canonical64KiB/nestedtypedbinding/privateACL/정확1namespace/원2IDs를대조한다. source565 전체4검사/설치65회귀·원source563 receipt 별도process/4typedfailure 보존PASS. linked등록현재파일/삭제권한/역사소유권검증은아니며후속두경로대조로분리한다.

RegisteredPairInspection은사용자가명시한두경로의원registered/receipt readonlycaps를동시에보유하고원전체canonicalrecord+seal을대조한다. source567 전체4검사/설치68회귀/별도source563원producer 대조/4typedfailure PASS. 각각유효한다른receipt혼합은실제nativeunit에서InvalidData로거절하며copiedfreshfixture에는그검증을소급하지않는다. 순차관측이며atomic snapshot/역사ownership/삭제authority는아니다.

RegisteredRemoval은 별도 명시 요청 factory에서 원 registered/receipt 대조를 consume하고 immutable expected IDs와 원 ReadRoot를 유지한 채 original-root-relative DELETE caps를 취득한다. disposition 전 full preflight, unknown 보존, 원HANDLE payload→metadata→빈 root, failure의 원remainingcaps retry를 제공한다. source570 Windows 전체4검사/설치75/실제4묶음·7행/외부receipt4 nativeID 대조 PASS. readonly inspection과 persisted JSON은 자동 삭제·historical ownership 권한이 아니며 CLI 연결과 active/upgrade/clean OS 인수는 별도다. 최대64개 묶음의 repeated SHA는 O(N²), 약16GiB 추가 읽기 가능성이 있으며 hard OS deadline/secure erasure 보장은 없다. 신규dependency/lock 변경 없음.

현재 일곱 명령 개발 CLI의 외부receipt 생성→별도 두경로readonly 대조→명시 정확묶음 제거를 source570 Windows 전체4검사/설치75/parser4/실제28명령/현재복사중제어2/native18ID root·독립대조로 인수했다. 명시 remove 이외 inspect/startup은 삭제하지 않는다. stdout phase·mutation·accepted dispositions·pre-dropcap관측과 afterexitcap없음을 구별한다. 일반activeinstaller·upgrade/역사ownership/서명trust 게이트는 남고 ADR 전체 Proposed는 유지한다.

ExplicitActiveSelection은 caller가명시한이미검증된등록+receipt pair를새private root의boundedcanonical marker에선택하는개발용 consistency후보다. 재시작은active/등록/receipt세경로를모두명시해원ID/SHA/namespace를readonly대조한다. 저장path에서자동탐색/launch/삭제authority를만들지않으며 기존active교체/upgrade/journalrepair는후속이다. 원root/FileIDs와domain SHA·selectedexe를결합하고 같은SID위조·historicalownership/publishertrust를보장하지않는다. Windows synthetic 선택6cases·별도process readonly대조 인수는완료했으며 정식installer/upgrade는후속이다. ADR Proposed를유지한다. 신규dependency/lock변경없음.


OwnedSelectedProcess는 명시3경로 readonly선택검사의 원caps와exe/ancestor핀을보유한Windows실행후보다. 원child를suspended생성하고원Job에bind한뒤childimage nativeID/SHA를resume전에대조한다. 종료는Natural/Cancelled/TimedOut와terminal_unconfirmed를구별하며resume_attempted/may_have_executed를성공resumed와구분한다. Windows 실제3case의원process 실행·종료 인수는완료했으며 실제GUI/정식installer는후속이다. ADR0005 Proposed를유지한다.

## run-selected 후속 구현 범위

`run-selected`는 사용자가 명시한 active·등록·외부 receipt 세 경로를 검사한 원 capability에서 선택 앱을 감독 실행하는 개발 명령이다. `--startup-timeout-seconds`는 기본300초, 1–600초이며 검사·생성·원 child image 검증·Resume까지 적용한다. 시작 후 `wait_cancelled`는 runtime deadline 없이 자연 종료 또는 취소를 기다리고 원 Job·exe·선택 pins를 supervisor 수명 동안 유지한다. nominal100ms poll은 OS 호출·스케줄링·정리 시간을 포함한 물리 취소 지연 보장이 아니다. supervisor 종료/Drop은 terminal 확인이 아니며 영구 detach·정식 installer를 제공하지 않는다.

새 MSVC target 전체 fmt/clippy/test/release는 완료했다. 테스트 집계760 PASS/0 FAIL/13 ignored이며 source576은 이전 같은 파일 수의 snapshot과 다른 정확 hash map이다. CPU c9f09dc0…8,709,632B, CLI ab8a6cde…795,136B다. 이전 MSVC CLI786,944B 대비8,192B 증가했고 CPU 크기는 같다. 신규dependency/lock 변경은 없으며 같은 CPU 크기가 동일 binary SHA나 과거GUI 비용 검증을 뜻하지 않는다. 실제 시작2초 기한을 넘긴 실행4초 이상 유지·자연 종료·취소·밝은 테마 재시작을 root/독립 검토했다. 새 CLI 실제17명령/14사례·원nativeID10·원파일 보존도 통과했다. 기존 bounded 실행의 새 바이너리 회귀는 실제15명령/13사례·GUI4까지 root/독립 PASS다. 부모 종료 후 자식만 살아 있는 분기는 새 source에서 별도 검증 중이다. [현재 전체검사](../validation/evidence/windows-msvc-run-selected-corrected-workspace-20261006-continue/result.json). 정식 설치·upgrade·서명·publisher/legal/cleanOS와 P0 전체 완료는 미주장이고 ADR0005는 Proposed를 유지한다.

원 Job 관측을 위해 추가한 active_process_count는 원 process creation과 retained Job HANDLE의 순간 count만 반환한다. 새 descendant-only fixture는 source577 전체 MSVC4검사 및 실제12명령/3case·nativeID30·원파일21을 root/독립 인수했다. 원Job의 livecount3과 자연대기10026ms를 관측했으며 이전 run-selected 인수를 소급하지 않는다. 부모가 먼저 exit17이면 자식 취소/기한의 reason은 달라도 부모 exit17을 보존한다. Job 종료 요청126을 이미 끝난 부모의 exitcode로 보고하지 않는다. count만으로 terminal/PID 소유권을 주장하지 않으며 이 변경도 ADR 전체 채택이나 정식 installer 완료가 아니다.

## 새 개발 설치 묶음의 원 capability

FreshUserBundle은 명시한 새 outer root와 registered/receipt/active 고정 자식을 원 생성 HANDLE로 유지하는 개발 coordinator다. source preflight 이후 outer를 생성하고 등록·외부 receipt 대조 이후 active를 마지막으로 게시한다. 기존 경로 채택·교체·Drop 삭제·durable journal 회수를 제공하지 않는다. 실패의 stage와 mutation 가능성은 capability 보유와 별도이며 preflight 실패에는 원cap이 없을 수 있다. UserBundleFailure의 boxed private 상태는 실제 보유 원cap만 보고한다.

Fresh readonly observer는 원 FreshRoot에서 LIST/ATTR/READ_CONTROL/SYNCHRONIZE 부분집합 HANDLE을 복제하고 원root-relative 파일의 immutable expectedNativeID를 대조한다. 쓰기·삭제·생성 권한은 없고 SHAREALL은 원 fresh writer cap과의 호환 정책이다. 일반 cold inspection의 shareREAD 정책은 유지한다. namespace·canonical·nativeID·전체SHA 대조는 순차 관측이며 역사적 소유권·publisher trust·atomic snapshot을 증명하지 않는다. 새 생성 root에는 LIST 권한을 포함하지만 observer 복제 시 WRITE/DELETE는 제외한다.

source579 corrected 전체4검사·774/0/13은 실제 root/독립 검토 완료다. 이후 capability 보고 교정의 새 전체 검증과 실제 prepare-user-bundle/재시작/복사 중 제어 인수는 진행 중이다. 신규 dependency/schema/lock 변경은 없다. 정식 installer·upgrade·rollback·clean OS·서명 및 전체 P0 완료와 구분한다.

명시 outer 하나의 UserBundleInspection 재시작 검사를추가한다. 원readonlyouter와기존selection/pair caps를보유하고정확3children namespace 및기존canonical/ID/SHA를전후재검증한다. inspect-user-bundle은개발용읽기검사이며자동발견·repair·trust·active설치·삭제·정식installer가아니다. 현재전체/native인수진행중이고ADR0005전체Proposed는유지한다. 저장schema/의존성/lock 변경없음이다.

실험 중인 install journal은 bounded immutable intent/단계 prefix와 원 HANDLE 재검사로 현재 상태를 관측한다. 실제 bundle manifest/active record 대조 없이 Active fingerprint를 commit으로 인정하지 않는다. caller 단계 SHA는 과거 실행 증명이 아니며 crash 이전 소유권·삭제 권한을 복원하지 않는다. Windows native 검증과 실제 crash 인수는 미완료다. 이 추가 실험으로 ADR 전체를 Accepted로 바꾸지 않는다.

추가 실험은 실제 Prepared/Registered/Receipt/Active 검증 직후 기록하는 fresh journal-aware coordinator다. root는 caller 취소·기한을 받는 private Prepared verifier와 원 writer bytes 재검사를 조정하고 기존 public API와 Noop 비용을 유지한다. 실제 Windows 전체/단계별 실패 인수는 진행 중이며 이 구현으로 ADR 전체나 정식 설치·crash recovery를 Accepted로 간주하지 않는다.


Native development bootstrap 후속 후보: 기존 worker-only local prepare/inspect service를 Windows 표준 폼의 별도 dlpad-install 바이너리에 연결한다. 네 명시 경로·실제 journal coordinator·single final mailbox1·active-only75ms timer·원worker 종료 후 CloseWait를 사용한다. UI I/O/shell/웹 UI·자동탐색·overwrite·launch는 도입하지 않는다. 기본Windowsfont/systemtheme를 사용하며 제품 CPU렌더러를 변경하지 않는다. 기존 windows-sys에 Gdi/LibraryLoader/Controls 선언feature를 추가하고 CPU에서 의존성을 명시활성화했으나 새crate/lock 변경은 없다. root전체빌드/실제GUI 인수 이전에는 채택완료로 표현하지 않으며 Proposed를 유지한다.


## 명시 개발 bundle 제거의 원 root 보존 후보

전체 결정은 Proposed다. 별도 explicit 요청은 현재 완전한 bundle/journal 일치를 readonly 관측한 뒤 canonical metadata와 원 ID를 빌린 상태에서 만든다. 검증 오류에는 원 inspector를 보유하고, 성공한 이동은 원 outer/registered/receipt/active ReadRoot만 이전한다. READ-only child 공유를 해제한 틈의 변경은 이후 DELETE 원ID/ACL/SHA/정확 namespace preflight에서 거절한다. DELETE 획득 실패는 boxed 원 ReadRoot와 오류를 반환하고 State에 미처리 roots도 유지한다. 역사적 권한이나 pathname 기반 삭제 fallback은 없다. active 철회 뒤 payload와 등록 metadata, receipt, 빈 roots를 처리하며 부분 실패 retry는 이미 처리한 root를 건너뛴다. 외부 journal/source/data/media/tools는 보존한다. 같은 production loop의 noop hook으로 중간 오류 회귀를 준비했다. 실제 Windows 제거 인수 전이며 [현재 작업](../work-items/S07-windows-explicit-user-bundle-removal.md)을 따른다. 신규 dependency/lock 변경은 없다.

2026-10-06 제거 엔진은 실제 Windows 전체840/0/13·4case public factory·추가native16명령 인수를 완료했고 개발 CLI16은 전체841/0/13·실제CLI36행/45명령·root추가native20명령으로 현재 부재/원자료 보존을 root/독립 검토했다. 앞의 후보 문단에서 미완료로 기록한 native engine 범위는 이 근거로 scoped 완료다. UI는 별도 Verify→inline confirmation→worker 새authorization을 사용하며 modal 추가와 foreground 전환을 도입하지 않는다. 한 worker/cap1·active-only75ms·원JoinHandle ACK를 유지한다. unpublished logical pending ReadRoot API/CLI 명칭을 unconverted_bundle_read_root로 명확히 했으며 OS HANDLE총수는 추론하지 않는다. 새 dependency/lock 변경 없고 CLI16 release는 직전 engine snapshot보다39KiB 증가했다. UI 비용/실제private 화면·정식installer·upgrade·cleanOS/physical·신뢰/서명은 미완료이고 ADR 전체 Proposed를 유지한다. [현재 UI](../work-items/S07-windows-native-development-removal-ui.md).

2026-10-06 source592의 개발 제거UI는 전체844회귀·실제11GUIcase/22실행·root73native명령/18직접PNG로 scoped인수했고 CPU0bf43b 18측정창의최대단일core0.624460%/WS71,757,824B 및합성30k/3DB를root/독립대조했다. 이binary범위를새source에소급하지않는다. 다음LocalInspectionRequest는두명시root만사용하며기존4path API도동일private inspector로위임한다. idle readonlyVerify는startup I/O/삭제권한이아니며새창검증성공후별도2root제거확인과freshauthorization을유지한다. 원 source/manifest는create-only설치입력에만필요하다. 현재서비스·UI·별도private process재시작fixture를통합검증중이며새dependency/schema/lock없음. 전체ADR Proposed/정식installer·upgrade·trust/서명·cleanOS/physical 게이트는유지한다. [후속범위](../work-items/S07-windows-native-existing-bundle-reverify.md).

## 2026-10-12: 원 HANDLE의 non-replacing publication

별도 NTFS 합성 실험의 Win32 두 buffer 변형은87 실패했고 NT class10 두 변형은 fresh rename·충돌·원ID/bytes/partial 보존을 통과했다. 이를 근거로 기존 개발용 active 선택 기록의 final 공개에 NT 원 HANDLE adapter를 연결한다. 임시 이름에 완성·검증한 canonical record만 final 이름으로 공개하고 기존 기록을 덮어쓰지 않는다. schema1/native marker ID·상위 retained mutation/caps 정책을 유지하며 after-call 불확실은 삭제 권한으로 해석하지 않는다. Windows Rust targeted131회귀를 실제 통과했으나 전체/CLI 인수는 후속이다. 이 구현 보완은 정식 설치·upgrade/rollback의 Proposed 범위를 사용자 확정 또는 완료로 바꾸지 않는다. [원본 실험](../work-items/S07-local-upgrade-publication-experiment-20261012.md), [Rust 통합](../work-items/S07-windows-fresh-publication-adapter-20261012.md).


2026-10-13 UI 후속: 사용자별 설치 위치 추천값은 readonly startup hint로 구현한다. packaged-discovery는 create-free로 유지하며 명시 target/journal을 덮어쓰지 않는다. 기본값과 FreshRoot·등록·삭제 authority는 분리한다. 새 의존성/lock/schema는 없고 bounded metadata 조회 비용은 작업에 기록한다. 실제 Windows 통합·설치 UI 검증은 진행 중이며 ADR 전체 Proposed를 유지한다. [작업](../work-items/S11-default-install-location-20261013.md).

2026-10-14 로컬 revision 순수정책 실험(Proposed): generation/predecessor와등록·receipt·exe의정확한from/to binding을갖는record를기존immutable active v1과별도도메인으로검증했다. canonical64KiB·unknown/duplicate/partial/future·overflow·stale head·cancel·잘못된rollback을거절하고원전이자료를보존한다. 실제Windows6/0/0과Root원source/provider/lock/raw검토는순수정책범위다. 현재Activated는proposed record상태이며nativecommit·launcher·삭제/실행권한을뜻하지않는다. durable단일writer/새namespace/확정또는불명commit/coldrecovery/앱데이터호환·실제rollback은아직미구현이다. 원active v1/journal/rootCargo/schema는변경하지않았다. 앱updater는기능및Git/CI후라는사용자순서를유지한다. [실험 기록](../work-items/S07-local-revision-policy-experiment-20261014.md).


### 2026-10-14: 삭제 후 같은 위치의 새 journal 선택 후보

사용자 재설치 MISSING 보고를 조사했다. 기존 explicit 제거가 외부 journal을 보존하는데 packaged 파생경로가 한개여서 presence Existing→Verify NotFound로 막힌다. 원기록 삭제나 journal 재사용을 하지 않고 packaged worker에서 기본경로와 숫자 suffix 총32 후보를 검사한다. target 부재는 최초 vacant journal, 기존target은 current bundle과 정확한 native 일치가 유일한 journal만 허용한다. 별도 영구 schema/새 trust 또는 삭제권한은 없으며 실제 service의 fresh create와 재검증을 유지한다. 10초 deadline/취소/동일text 재설정 guard, 후보상한 및 기존explicit4path 계약을 유지한다. native 설치→제거→재설치→cold 검증은 진행 중이고 전체 ADR Proposed 상태는 유지한다. [현재 작업](../work-items/S11-reinstall-location-20261014.md).


## 2026-10-16: 제안 기반과 지원 정책 시험의 구분

전체 결정은 Proposed다. 별도 test-only cold journal v4는 기존 원 parent HANDLE과 정확한 leaf의 NtCreateFile FILE_OPEN으로 append 권한을 관측하고 Windows17회귀·strict engine clippy를 통과했다. 앞선 ReOpenFile의 raw5 실패는 보존하며 Windows 전체의 일반적 불가능성으로 단정하지 않는다. 이 실험은 Live 제품/설치본에 채택하지 않았다.

최초 P0의 실제 복구·업그레이드 시험은 현재 지원하는 새 폴더 설치·Accepted ADR0009 명시 감독 실행 정책으로 진행했다. 서로 다른 두0.1.0 개발 snapshot A31/B32의 동일 합성 프로필 A→B→A와 B 설치 충돌 후 A 재실행을 실제 확인했다. Queued3·library2·작은 테마 변경의 좁은 호환 근거이며 미래 DB downgrade·자동 교체/rollback·crash/powerloss 보장이 아니다. [실제 범위](../work-items/S07-manual-side-by-side-profile-20261016.md). 전체 durable revision 제안을 사용자 확정 P0 의무로 상향하지 않으며 앱 updater는 사용자 지정 후속 순서를 유지한다.

## 2026-10-17 동일 폴더 공개의 Windows 호환성 수정 후보

실제 hosted Server2022의 원 파일 공개가 NTSTATUS0xc0000043으로 실패해 설치 계열21회귀로 전파됐다. 동일 폴더의 단순 leaf 공개는 원 DELETE file HANDLE을 유지한 채 FILE_RENAME_INFORMATION.RootDirectory=NULL을 사용하도록 수정한다. 원 fresh root identity 검증·replace=false·foreign write/delete 거절은 유지하며 절대경로 재개방/공유 모드 완화로 대체하지 않는다. [공식 구조 계약](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/ns-ntifs-_file_rename_information)과 [원 실패·최소 비교·실제 검증 상태](../work-items/S11-hosted-windows-file-sharing-20261017.md)를 따른다. 이 국소 호환성 수정은 Proposed durable revision/updater 아키텍처의 채택을 뜻하지 않는다.
