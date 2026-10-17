# 도메인과 불변 조건

앱 업데이트의 Signed candidate는 고정 키의 원 bytes 서명과 버전/sequence/채널/유효기간 정책을 통과한 관측값이다. 설치·실행 권한이 아니다. Staged update package는 Windows create-only 저장과 원 hash/identity를 확인하고 readonly reader·parent pin을 보유한 결과이며 자동 앱 교체를 뜻하지 않는다. 생성/공개/정리 불명 상태는 성공이나 삭제 권한으로 해석하지 않는다. [ADR-0012](docs/adr/0012-signed-app-update-candidates.md)를 따른다.

개발 검증의 scan barrier는 제품 상태가 아니다. 기본 OFF fixture에서 첫 실제 일반 파일 관측 INSERT 이후, publication 전 원 이벤트를 기다려 CloseWait·취소 rollback·worker 반환 ACK의 순서를 관측한다. 일반 scan의 협조적 취소 경계와 기존 성공 목록 보존 계약은 그대로다. [Windows 인수 범위](docs/work-items/S03-windows-scan-closewait-native.md)를 따른다.

| 용어 | 의미 |
|---|---|
| Source | 원본 URL, extractor, 원본 영상 ID. 영구 식별에 만료되는 미디어 URL을 쓰지 않는다. |
| Download job | 원본에서 로컬 결과를 만드는 사용자 요청. 재시도해도 ID가 유지된다. |
| Attempt | 한 번의 외부 프로세스 실행. 실행별 ID, 도구 버전, 오류를 기록한다. |
| Artifact | 완성된 영상·오디오·자막 등의 실제 파일. 한 job에 여러 개 가능하다. |
| Library entry | 저장 폴더에서 관측한 파일과 알려진 출처. job 없이도 존재한다. |
| Partial | 이어받기에 쓰는 미완료 파일과 상태. 완성 파일 목록과 구별한다. |
| Tool installation | yt-dlp, FFmpeg/ffprobe, JS runtime 등 실행 도구의 설치 기록. |
| Managed / External | dlpad 전용 디렉터리에 앱이 설치함 / 사용자가 별도로 설치함. 삭제 권한이 다르다. |
| Upload job | 특정 artifact를 YouTube에 전송하는 별도 작업. 다운로드 성공과 독립적이다. |
| Pause | 프로세스 동결이 아니라 안전한 중단 후 재개 준비. 소스에 따라 재개가 불가능할 수 있다. |
| Background | UI 창 없이 같은 앱이 작업을 유지하는 모드. OS 로그아웃·종료 후 지속 실행은 아니다. |
| Diagnostic report | 특정 Attempt의 정제된 증거와 판단·생략 항목을 묶은 전달 자료. 실행 권한은 없다. |
| Known issue match | 오류·구조·버전 조건과 외부 근거의 일치 결과. 유사 사례와 원인 확정은 구별한다. |

불변 조건:

- UI는 프로세스·DB를 직접 제어하지 않고 application command를 보낸다.
- 상태 변경은 저장 후 알린다. 진행률 샘플은 영구 이력이 아니므로 합치거나 버릴 수 있다.
- `Downloaded`는 검증한 최종 artifact가 존재할 때만 성립한다. 100% 네트워크 전송과 같지 않다.
- 라이브러리는 현재 파일의 관측이고, 이력은 앱이 수행한 작업의 기록이다. 폴더만으로 과거 URL·실패를 복원하지 않는다.
- 외부 설치 도구와 사용자 미디어는 자동 삭제하지 않는다.
- 다운로드 성공 후 업로드 실패가 발생해도 로컬 파일은 보존한다.
- 인증 값은 UI 상태·로그·명령 미리보기·일반 DB에서 제외한다.
- 진단·AI 추정은 다운로드 상태를 변경하지 않는다. 외부 이슈 종료와 설치 버전의 수정 적용은 별개다.

개발 구현 용어:

- Publication plan: Verifying에서 저장하는 소유 staging/final 경로·크기 계획. Unix에서는 두 파일 identity가 동일해야 크래시 뒤 Downloaded로 복구한다.
- Dirty hint: native 파일 이벤트를 합친 고정 flag. 라이브러리 전체 관측의 진실은 성공한 reconciliation transaction이다.
- Instance guard: 앱 전용 data 디렉터리 잠금. Linux의 안전한 private 복귀 endpoint가 준비된 경우 두 번째 실행은 기존 창 표시만 요청하고 종료한다. endpoint 미지원·준비 실패에는 중복 실행을 거절한다.

이 구현 용어는 [ADR-0002의 검증 기록](docs/adr/0002-runtime-and-storage.md)에 연결한다. bounded tail 로그는 시작 로그를 잘라낼 수 있으므로 보고서에 truncation을 남긴다. Downloaded는 tool exit=0, final marker와 소유 경로·regular/nonempty 파일 확인을 모두 거친 경우다. 미디어 decode 검증은 아직 별도 게이트다.

## 개발 adapter의 관측과 저장 위치

ProgressFeed는 실행 중 작업의 최신 다운로드 속도/ETA 한 개를 전달한다. 3초 이상 지난 관측은 unknown이며 완료 증거가 아니다. 선택 인터페이스의 OS 수신/송신은 다른 앱 트래픽을 포함한다. 저장 루트는 app data의 settings에 유지하며 실패한 선택은 기존 값을 보존한다. WatchSession은 알림 재연결, root 수정 시각과 30초 재확인으로 DB reconciliation을 요청한다. Windows native library snapshot 후보는 원 directory HANDLE의64KiB batch에서 name/size/FILETIME/opaque128bitID를 관측한다. root/child directory ID·volume·reparse·finalpath를 검증하고 EOF 뒤 원rootpin을commit까지 유지한다. 관측 전 지원preflight가 미지원이면 기존 보수scanner를 사용하며, 실제관측중 오류는rollback이다. 이 값은 관측시점의 목록정보이며 현재fileHANDLE/삭제·실행 권한이 아니다. 유일한rename 후보의 레코드 이전에는 원perfile path/metadata/nativeID를 다시검증한다. Windows 실제회귀·전체ID 대조는 통과했고 최신GUI 성능 인수는 진행중이다.

외부 FFmpeg 선택은 Download job의 옵션 snapshot에 속한다. 확인한 canonical 경로/버전은 Attempt component 증거이며 외부 설치의 소유권을 변경하지 않는다. 빈 선택은 yt-dlp의 PATH 탐색이며 앱이 확인한 FFmpeg 버전으로 표시하지 않는다. 정확 자르기는 구간 요청이 있을 때만 재인코딩 옵션을 적용한다.

## 작업 수명 감시와 시작 복구

작업 이력 화면의 읽기 조회·새로고침·다음 페이지는 시작 복구 준비 상태와 독립된 읽기 행동이다. 복구 실패에서도 오류와 기존 기록을 확인할 수 있어야 한다. 작업 재개는 실행 행동이므로 시작 복구 ready, 작업/닫기 gate를 통과해야 한다. 조회 가능성을 새 Attempt 생성 권한으로 해석하지 않는다. 읽기 worker는 원 DB를 생성·수정하지 않으며 읽기 실패를 빈 목록으로 표현하지 않는다.

Guardian은 Linux 외부 도구 실행에 한정한 headless subreaper다. 앱과 연결된 anonymous socket의 단절은 작업 owner 소실이며, guardian은 소유 leader/입양 자손의 PID가 재사용되기 전에 종료·회수한다. 상시 daemon 또는 다운로드 재개 worker가 아니다. 시작 복구 준비 상태(확인 중/ready/실패)는 journal 대조와 새 작업 시작 gate이며 job의 Interrupted/Downloaded와 구분한다. 파일 증거 확인 없이 Interrupted를 Downloaded로 바꾸지 않는다.

작업 이력 재개는 원본 Job ID와 검증된 옵션 snapshot을 유지하고 새 Attempt를 만드는 명시적 행동이다. 완료·취소 작업은 재개 대상이 아니며 legacy 옵션을 argv에서 추측하지 않는다. File identity는 현재 Unix dev/ino 관측치이며 조건부 외부 rename 정합성에만 사용한다. 영구적인 콘텐츠·출처 identity로 표현하지 않는다. 진단 보고서는 세션 메모리 자료이며 자동 disk cache가 없다. 명시 export는 사용자 소유다.

일시 중단 의도(Pause intent)는 cancellation보다 먼저 전달하며 소유 프로세스 종료 ack가 확인된 때만 Paused를 저장한다. 도구 실패와 자연 완료는 pause 성공으로 바꾸지 않는다. 취소된 Job의 partial 보존은 재개 권한과 다르다.

완료 증거 묶음(Artifact batch)은 adapter가 검증한 최대 3개의 결과다. core의 complete_many는 전체 경로·크기를 먼저 검사한 뒤 한 번의 revision으로 Downloaded를 적용한다. 일부 항목이 잘못되면 상태와 기존 증거를 변경하지 않는다. 영상·자막의 실제 publication/recovery 연결은 이 정책만으로 완료되지 않는다.

## 협조적 목록 관측 중단

Scan cancellation은 폴더 관측 worker가 directory entry 및 DB publication/commit 직전 경계에서 종료 의도를 확인하는 정책이다. 관측중취소는transaction rollback으로마지막성공snapshot을보존한다. 개별filesystem/SQLite호출중즉시중단은보장하지않으며최종확인뒤commit의경쟁에서는완료가이길수있다. UI의CloseWait는소유download/scanworker의완료또는중단ack를기다리는상태이며 Download job state가아니다. 새작업/저장위치변경을차단하고unfinishedthread를UI에서join하지않는다. workerpanic은고정오류와stopack를전달하며정상scan성공으로표시하지않는다. [구현 계약](docs/work-items/S01-library-scan-cancel.md)과 [후보 연결](docs/work-items/S01-cpu-library-integration.md)을 따른다.

## Linux 사용자 서비스 보호

User-service guardian은등록된Linux앱의기본작업소유권경계다. 기존사용자systemdmanager가고유transientunit/cgroup안에서동일앱의headlessguardian을실행한다. launcher는앱의직접Child이며다운로드worker나재개daemon이아니다. Invocation identity는동일managerbus의AddRef와16byteInvocationID/cgroup/MainPIDstart/pidfd를함께검증하는일시소유권증거다. PID숫자만으로소유권을재구성하거나미확인unit을중단하지않는다. capability부족은작업시작실패이며weakfallback이아니다. 독립engine개발helper의directfixture와제품등록앱의기본보호를구분한다. [부분결정과실증](docs/adr/0002-runtime-and-storage.md)을따른다.

Startup permit은Linux guardian이ready되기전launcher의정리소유권을예약하는최대8개슬롯이다. 정상ready/미spawn에는반환하고,기한내회수되지않은Child는단일Startup reaper에소유권을이관한다. reaper는앱내부thread이며daemon/다운로드재개worker가아니다. empty일때Condvar로대기하고Child를버리거나미확인PID를kill하지않는다. [인수범위](docs/work-items/S03-guardian-startup-wait-implementation.md)를따른다.

## 세션 쿠키 실행 권한과 정리 보류

Session cookie lease는 검증된 Netscape 파일의 앱 소유 private 복사본에 대한 실행 전용 권한이다. 일반 DownloadOptions/snapshot/DB에 저장하지 않고 metadata·다운로드에만 전달한다. 인증 실행의 원시 출력과 자유 metadata는 외부 반환 전에 생략하며 내부 artifact 검증은 유지한다. 작업·재개마다 새 선택을 요구한다.

Deferred cookie cleanup은 프로세스 정리가 미확인인 경우 복사본을 남기고 lease FD를 닫는 상태다. Download job 성공·취소 상태나 reader 종료 증거가 아니다. 보류된 lease는 다시 실행에 사용할 수 없으며 앱 종료·PID·파일명·mtime만으로 삭제하지 않는다. [진행 중인 계약](docs/work-items/S03-process-cleanup-outcome.md)과 [회수 검토](docs/work-items/S02-cookie-startup-recovery-review.md)를 따른다. guardian/service terminal 확인·시작 시 회수·비Unix ACL의 미완료를 그대로 유지한다.

Terminal ownership acknowledgment는 도구 exit code와 독립적인 소유 트리 정리 확인이다. Linux guardian의GC는자식회수확인,GU는미확인이다. 사용자서비스launcher는invocation/cgroup정리뒤LC를발신하며실패는LU다. direct개발fixture는GC+EOF,service는GC+LC+EOF를유한시간에확인해야한다. 누락/중복/역순/미지프레임은TerminalUnconfirmed이고인증파일정리를보류한다. 도구의정상exit126과보호실패를stderr로추측하지않는다. [실제22case와남은인수](docs/work-items/S03-process-cleanup-outcome.md)를따른다.

Cookie recovery record는Linux의private복사본·record·marker와cache/owner nativeidentity,OSnonce,recorddigest,kernelbootUUID를묶은최대4KiB자료다. 쿠키값/원본path/URL은기록하지않는다. 검증된이전boot만reader종료증거로사용하며같은boot는PID/unit명/mtime으로삭제하지않는다. scan/import는128개상한이고foreign/partial/변경항목은보존한다. [첫 scoped검증](docs/work-items/S02-cookie-recovery-records.md)은실제reboot/동일bootreceipt인수가아니다.

Linux Cookie journal anchor는 private owner directory와 recordstate.json의 native device/inode를 durable recovery record2에 바인딩한 값이다. Recovery marker digest는 이 anchor도 포함한다. record1은 journal 없는 기존 포맷이며 읽기 호환을 유지한다. Reader slot의 Intent/Bound/Resolved와 독립 cleanup receipt는 실제 파일 검증 기반이며, Resolved memory만 정리 근거로 사용하지 않는다. 초기 anchor snapshot은 import·정상 정리·빈 journal의 prior-boot 복구만 검증했고 same-boot 기록을 보존했다. 이후 제품 I/B launcher gate와 native receipt writer를 연결했다. 원 Bound/Resolved가 하나 이상이며 실제 receipt가 모두 확인되는 same-boot 기록만 회수하고, 빈 journal/Intent/미완성 receipt는 보존한다. PreviousBootAuthority는 현재 kernel UUID를 직접 캡처하고 삭제 전 다시 확인하는 별도 권한이다. 검증된 원 record/marker 및 journal의 모든 slot boot/digest가 이전 부팅에 묶인 경우만 pending을 회수하며, 상태를 Resolved로 변경하거나 receipt를 만들지 않는다. 실제 kernel 재부팅 시험은 아직 수행하지 않았다.

Linux gated cookie reader는 metadata0/download1의 일회성 journal slot을 사용하며 download1은 metadata0 Resolved 뒤에 시작한다. 새 worker는 새 session lease를 가져오며 같은 lease에서 slot을 초기화하거나 재사용하지 않는다. Reader container는 native cgroup 수명을 유지하는 전용 nonce top-level slice이고, reader service는 그 안에서 Begin을 기다리는 소유 guardian이다. reader service, auxiliary parent, EOF keeper를 한 transaction으로 생성한다. EOF keeper는 parent 밖 app.slice에서 launcher만 가진 pipe writer의 EOF를 기다리며 parent 의존성을 유지한다. reader는 RemainAfterExit=false로 원 종료값을 보존한다. 원래 parent mount/device/inode와 service Invocation을 고정하며 native pidfd 종료·populated0·빈procs 뒤에만 receipt를 sync한다. receipt sync 뒤 keeper writer를 닫고 정확한 keeper의 종료·원 pidfd를 확인한다. keeper와 reader의 소유 서비스만 정리하고 parent는 Unref만 한다. 부모 직접 Stop은 사용하지 않는다. app은 C→I→durable Bound→B 순서로 진행하고 GC+LC+EOF 뒤 실제 receipt resolve sync까지 성공해야 pending을 해제한다. journal mutex는 bounded 파일 검증/쓰기 동안만 사용하고 프로세스 실행 동안 보유하지 않는다. 제품 adapter의 실제 합성 성공·metadata 실패·download 실패·취소4case와 같은 boot의 앱 종료/강제 종료 뒤 회수2case를 검증했다. 초기 accepted start, GetUnit 뒤, Invocation 뒤 실패를 실제 사용자 manager에서 시험해 도구 실행 0·Intent 보존·소유 세 unit 회수를 확인했다. 실제 제품 foreign live/queued 서비스 보존2case와 최신 기본/CPU 전체 검증을 통과했다. 다른 OS와 실제재부팅/전체P0 인수는 남는다. EOF keeper는 인증 reader마다 helper 프로세스 하나와 transient service 하나, pipe FD 두 개를 추가하며 실행 비용은 별도 측정한다.

## Windows 인증 reader gate

Windows gated cookie reader는 metadata0/download1의 원 native 파일 ID·session nonce·base digest와 guardian/tool FILETIME을 원 durable journal에 묶는다. 앱이 원 helper HANDLE의 생성 시각과 Prepared를 대조하고 Bound를 sync한 뒤 journal 잠금을 넘긴다. helper의 독립 잠금 ACK 뒤 Begin을 보내며, 원 receipt writer는 exact capability 전달 후 부모가 즉시 닫는다. Mutex는 짧은 원 파일 검증/commit 동안만 사용하고 도구 실행 중 보유하지 않는다.

Terminal은 원 Job의 ActiveProcesses0 확인 뒤 sync한 256-byte actual receipt와 원 helper 종료/이벤트 EOF를 함께 요구한다. pending 해제는 원 Bound와 실제 receipt 재확인·resolve 뒤에만 한다. pristine은 두 slot Empty이며 원 live import의 미사용 정리에만 쓴다. startup에서는 pristine/Resolved/PID/mtime/boot 추측만으로 삭제하지 않는다. private original marker는 원 6개 native IDs 및 marker 자신의 ID를 묶고, 회수는 원 namespace pins/ACL/strict5entries/journal lock/actual receipts를 재검증한다.

actual independent helper 정상·실패·큰 config와 app-like processkill, 제품 import에서 helper receipt→publicrecover 성공을 검증했다. 제품 downloader gated capture는 연결 후 native 회귀 검증 중이며 실제 GUI crash·3OS·실제사이트·전체P0 완료를 의미하지 않는다.

## 관리 archive package

ArchivePackage는 압축 원본과 component별 크기·SHA·역할을 구분한 version1 실행 공급 계약이다. Managed ownership 선언은 실제 소유 파일 identity나 공급 trust 증명이 아니며 설치 worker가 별도로 검증한다. Bounded ZIP extraction은 호출자가 제공한 원 read-only source와 새 output File capability로만 동작하며 게시·실행·삭제·active DB 전환을 하지 않는다. 실제 공급 승인과 관리 설치 사용자 흐름은 독립 게이트다. [계약](docs/work-items/S04-bounded-archive-contract.md)을 따른다.

## 비활성 관리 archive 설치와 선택

InstalledPackage는 검증된 ArchivePackage, 원 설치 디렉터리와 main InstalledTool의 세트다. 로컬 archive 설치 성공은 비활성 파일 세트의 create-only 게시이며 active DB 변경이나 runtime version 증명이 아니다. metadata 목록은 bounded manifest·파일 이름/type/native identity/실제 크기를 확인하고 큰 payload 전체를 읽지 않는다. 명시 선택·active 사용·제거는 전체 component digest를 검증한다. 설치·선택·제거·local import는 manager 간 같은 root lock을 공유하고 초기 원 root 디렉터리 pin 교체를 거절한다. v1 staging journal은 유지하며 v2는 component와 두 metadata의 최대10개 증거를 기록한다. 부분 final publication/삭제와 미확인 crash leftovers는 완전 rollback으로 표현하지 않고 보존/후속 게이트로 남긴다. [실제 통합 검증](docs/work-items/S04-package-native-boundaries.md)을 따른다.

## 선택 Deno 실행 증거

DownloadOptions의 optional Deno 경로는 version1 저장 옵션에서 누락 시 None으로 복원한다. 원 사용자 경로 snapshot과 실행 시 확인한 canonical 경로·버전 증거를 구분한다. metadata와 다운로드는 동일 runtime argv를 사용한다. Attempt의 deno component는 실행 전 실제 검사 증거이며 공급 신뢰나 외부 설치 소유권 증명이 아니다. 확인된 사용자 취소와 프로세스 정리 미확인은 별도로 분류한다. [작업 계약](docs/work-items/S04-deno-download-options.md).

## 압축 원본 검증과 TAR 매핑

TarComponentMapping은원TAR member와평면설치component의명시대응이다. TAR reader는경로를생성하지않으며caller소유빈File에선택bytes만쓰기전에전체header/type/크기를검사하고streamEOF까지확인한다. TarXzSourceSpec은원File의정확압축bytes/SHA와별도압축검증 gate다. singleXZ는per-block LZMA2메모리상한64MiB를사용하며프로세스RSS보장은아니다. source원namespace pin·공급trust·소유게시·부분outputcleanup·관리활성화는caller계약이다. [현재범위](docs/work-items/S04-bounded-tar-xz-gate.md).

## 명시적 TAR/XZ 관리 package layout

ArchivePackage version1은layout필드가없거나None이며기존ZIPflat256MiB로읽고저장시None필드를생략한다. version2는ArchiveLayout::TarXz의원root와member→component이름매핑을필수로갖고선택설치합384MiB를제한한다. mapping은component별SHA/bytes의별도authority를만들지않으며기존components를참조한다. TAR전체선언768MiB와stream800MiB는폐기검증한도이지디스크예약이아니다. 설치는같은원File/nativepin/rootlock/deadline/staging정책을사용하며성공해도비활성이다. HASH기반하위선택은runtime정확버전검증선택의대체가아니다. [공유계약](docs/work-items/S04-tar-xz-package-integration-contract.md).

## 관리 실행 변경 배제 기반

ExecutionLease는관리root의원install-transfer-lock을공유로보유하는capability다. manager의설치/선택/제거배타잠금과경합하며복수reader가가능하다. parentcapabilitydrop은명시unlock이아닌FDclose이며Unix복제reader가남으면변경배제가유지된다. 이는실행/guardian/서비스수명연결또는appcrash안전을아직증명하지않는다. TerminalUnconfirmed는lease scope종료를reader종료로해석할수없는상태이며[수명계약](docs/work-items/S04-managed-execution-lifetime.md)을따른다.

## 관리 실행 변경 보류 기록

ManagedExecution은sharedExecutionLease와create-only .execution-pending 원기록을결합한실행scope다. 완료된ownedreader정리뒤명시finish만기록을clear한다. Drop/appcrash/SessionCleanupDeferred/변경된기록은보존하며manager다시열기나PID추측으로삭제하지않는다. pending이존재하면모든관리mutation의exclusive권한은ExecutionPending을반환한다. 원options/resume저장snapshot의canonical관리경로에만worker가이scope를연결하며외부도구는자동관리하지않는다. 자동pending회수와guardian별capabilityhandoff는미완료[계약](docs/work-items/S04-managed-execution-pending-integration.md)이다.

관리 실행 pending version2는원create-onlymarker와retainedroot HANDLE의nativeIDs/sessionnonce를boundedJSON에기록한다. 원ID관측은reader종료/ACL/receipt권한이아니며자동회수를허용하지않는다. 기존version1/unknownpending도변경차단을유지한다. [구현범위](docs/work-items/S04-managed-marker-native-binding.md)를따른다.

ArchiveLayout::Zip는version2의명시source-member→평면component mapping이다. v1flat32entries/256MiB는불변이며v2ZIP는4096entries/전체선언768MiB/선택설치384MiB를분리한다. 실제decode/CRC/SHA는선택component에적용하고미선택header/local/range/path를검사한다. 공식Windows공급원자료와설치성공은license/source/notice공급채택을대체하지않는다. [통합계약](docs/work-items/S04-v2-mapped-zip-integration.md)을따른다.

Managed Windows reader record는최대8single-use stage의Intent/Bound/Resolved와원root/marker/lock/journal/receipt8개IDs를묶는16KiB상한자료다. Windows cookie domain과다른digest/receiptmagic을사용한다. 실제nativeoriginalreceipt권한이없는matchingbytes/Resolved상태는삭제증거가아니다. [통합범위](docs/work-items/S04-windows-managed-reader-records-integration.md)를따른다.

## Windows 관리 실행 영구 경계 (개발 통합)

Managed execution marker v3는 원 root·marker ID, 세션 nonce와 journal·8 receipt의 원 SessionAnchors 및 도메인 분리한 base digest를 고정한다. Windows 생성 경로는 관리 exclusive lock 아래 이 기록을 sync한 뒤 공유 실행 lease로 전환한다. v2 및 부분 기록은 자동 복구 권한이 없다. 현재 reader 실행 연결과 startup 회수는 아직 개발 중이다. pristine 정리는 같은 생성 객체에서 reader가 전혀 시작되지 않았다는 확인에 한정하며 모든 journal 파일의 실제 부재를 확인한 뒤 marker를 마지막에 지운다. [통합 작업](docs/work-items/S04-windows-managed-reader-lifecycle-integration.md)을 따른다.

Paired guardian binding은 managed와 cookie의 두 typed Bound를 같은 실제 guardian/tool creation FILETIME에 묶는 순수 전달 형식이다. metadata는 cookie slot0, download는 slot1에 대응하며 원 파일 domain alias와 변조된 nested serde를 거절한다. 이 형식 자체는 Job 종료·receipt 작성·정리 권한이 아니다. [순수 계약](docs/work-items/S04-paired-guardian-binding.md)의 4개 시험에 이어, [공개 합성 다운로드](docs/work-items/S04-paired-download-native.md)에서 실제 단일 Job과 두 원 journal/receipt 연결의 성공 경로를 확인했다. 실패·취소·부모 종료 및 startup 회수는 별도 인수이며 이 순수 형식만으로 삭제 권한을 재구성하지 않는다.

## Windows 관리 시작 복구의 확인 범위

ManagedExecutionRecovery는 원 v3 marker의 immutable anchors/base digest와 원 install lock, strict ACL/native namespace 및 실제 종료 receipt를 모두 확인한 뒤 소유 journal 9개를 정리하고 marker를 마지막에 닫아 부재를 확인한 결과다. recovered와 pending을 구분하며 absent/Busy/pristine/Intent/legacy/변조는 종료 증거가 아니다. 관리와 cookie는 독립 원 권한으로 회수한다. 공개 합성 paired 부모 종료 뒤 양 회수/DB Interrupted와 원 Job 자연 종료를 실제 Windows에서 확인했다. 앱 시작 worker와 기존 재확인 행동의 연결은 별도 최신 검증 중이며 외부 도구는 기존 관리 보류 정책과 독립적이다. [통합 및 증거](docs/work-items/S04-managed-startup-recovery-integration.md)를 따른다.


## 관리 Deno 실행 환경 정책 (통합 중)

Windows managed Deno environment는 고정 policy1의 DENO_DIR과 DENO_DISABLE_NODE_SHIM만 전달하는 typed 프로세스 정책이다. 경로 문법/환경 인코딩 검증은 cache native 소유권 증명이 아니다. helper 전달·정확 관리 Deno 분류·소유 cache와 실제 EJS 인수 전에는 production 선택에 적용하지 않는다. 기본 None은 기존 환경 상속이며 전달 미연결 Some은 journal 변경 전 거부한다. [현재 구현과 실패 인수](docs/work-items/S04-runtime-environment-propagation.md).


## Windows 관리 Deno 분류와 cache 원 권한

VerifiedManagedDeno는 활성 선택과 독립적으로 명시 Deno 경로의 전체 archive package/원 owner·component·record native identity와 digest/OS target을 검증한 읽기 권한이다. 외부 선택은 관리하지 않으며 내부 불명/alias/hardlink/reparse는 거절한다. OwnedDenoCache는 `.runtime-cache/deno`의 원 root/runtime/cache/owner/lock HANDLE·SID 보호 ACL·version1 owner record를 유지한다. 경로나 환경 policy만으로 이 원 권한을 재구성하지 않는다. 별도 cache.lock은 owner JSON 읽기와 byte-range lock을 분리한다. inventory의 32MiB는 사후 논리 합계 예산이며 실행 중 hard quota가 아니다. Windows 실제 기반 회귀는 통과했지만 생산 factory와 독립 guardian 수명 연결은 후속이다. [분류](docs/work-items/S04-managed-deno-classification.md), [cache](docs/work-items/S04-owned-deno-cache.md).

## Windows cache 인계 상태

AppHeld는 원 cache.lock의 app 자체 잠금과 검증된 원 pins를 유지하는 단계다. Released는 explicit unlock 뒤 잠금 권한이 없는 단계이며 verify/binding/실행 권한으로 쓰지 않는다. helper는 expected original IDs/owner SHA와 inherited lock을 검증한 뒤 Ack gate에서 자체 lock을 획득하고 Begin 직전에 다시 확인한다. wire나 DuplicateHandle만으로 Held를 만들지 않는다. HelperHeld는 원 Job 종료·receipt·terminal까지 유지하며 사후 논리 inventory 실패/예산 초과를 helper exit126으로 반환한다. 원 app HANDLE close 후 실제 helper-live Busy와 helper 종료 뒤 재획득은 Windows native로 확인했다. 부모 processkill/managed·paired Some/생산 다운로드 인수는 후속이다. [현재 범위](docs/work-items/S04-owned-deno-cache-native-handoff.md).

## 관리 Deno 원 authority의 제품 연결

WindowsManagedDenoEnvironment는 원 cache 권한과 VerifiedManagedDeno를 Arc로 보유하며 worker의 DownloadSession에서만 빌린다. DB/options에 권한을 직렬화하지 않는다. wire는 immutable 원 cache binding/lock HANDLE을 전달하고 새 parent authority를 만들지 않는다. 실제 Windows 합성 공개 다운로드5case와 원 receipts/cache 보존은 확인했다. 이후 감사에서 원 실행 root 혼합과 factory 후 패키지 변조의 journal 전 거절 보강이 진행 중이며, 기존 snapshot 성공은 최신 source 완료 증거가 아니다. 실제 공식 Deno/EJS/site·GUI/공급 인수는 후속 P0다.

## 관측 지표의 최신 표본과 표시 갱신

Metrics freshness deadline은 CPU native host의 일시적인 최신성 확인 기한이다. 일반 egui repaint 기한과 따로 보유하며 둘 중 빠른 기한으로 기다린다. 같은 표시키의 새 timestamp는 전체 frame 없이 mailbox에서 갱신하고, stale3초·실패·표시 변경은 redraw한다. Unknown의 건강한 worker는1초마다 복귀를 확인한다. PresentedVisibleKey는 실제 status가 렌더한 pending 키를 성공한 visible frame 뒤 확정한 값이며 timer/hidden/present 실패는 확정하지 않는다. persisted 상태·작업 실행 권한·새 thread가 아니다. 구현의 Windows 전체 검증790/0/13은 통과했으며 실제 private stale/복귀·성능 인수는 진행중이다.

볼륨 여유와 선택 인터페이스 지표는worker가1초마다관측하고 최신표본하나로교체한다. UI가잠시미소비해도이전표본을우선보존하지않으며, UI의읽기는경합시기존표본을유지하는try_lock이다. 표시된0.1GiB/0.1KiB값·선택revision·관측실패가변할때만worker가repaint를요청한다. UI는3초stale정책과worker실패·stopACK를별도로유지한다. 표시가같다는이유로표본timestamp를갱신하지않으면안된다. 이는작업실행권한이나persisted설정의새상태가아니다.

라이브러리DB의조회결과path는UTF8 128KiB, file_identity는128bytes까지Rust문자열로반환한다. 손상필드는오류이며조용히제외하거나절단하지않는다. 선택root의32KiB입력정책과각파일전체path는다른대상이다. SQLite내부index/CAST메모리전체의상한을뜻하지않고identity가삭제권한을부여하지않는다.

## UI의 IME 편집 transaction

IME editing transaction은Download job/domain상태가아니라focusedTextEdit의짧은버퍼상태다. 원committed문자열과directedselection을boundedephemeralbaseline으로보존하고emptyCommit/취소에서복원한다. nonemptyCommit은확정이며외부텍스트변경을오래된baseline으로덮지않는다. 조합중문자열은undo의committedcheckpoint와구별하고snapshot은DB/일반로그/serde/debug에저장하지않는다. [좁은채택](docs/adr/0001-native-ui.md)과 [실제회귀](docs/work-items/S00-egui-ime-transaction.md)를따른다.

## 별도 HTTPS notice 공급

ArchivePackage v3는 flat ZIP 실행파일과 별도 고정 HTTPS Notice 자료를 구분한다. notices 항목은 기존 components의 정확 이름·Notice 역할·bytes/SHA를 참조하며 revision40hex를 보관한다. archive_components는 외부 notice를 ZIP member 선언에서 제외한다. 최대2개/각1MiB/합2MiB이고 v1/v2의 empty notices는 저장에서 생략해 기존 JSON 계약을 유지한다. 자료는 같은 root lock·절대기한 아래 원 파일 capability로 검증하며 전체 세트 성공 이후 inactive 설치로 게시한다. 공식 공급 신뢰와 notice 내용 완비·자동 설치 채택은 별도 게이트다. [작업 계약](docs/work-items/S04-external-notice-package-transaction.md).

Windows native 목록의 관측 집합과 기존 root DB 집합이 정확히 같으면 publication의 UPDATE/UPSERT/DELETE를 생략하는 후보를 통합했다. root 전체행 수·유일 path·size/mtime/opaque identity/present 및 SQLite storage class를 대조하고 observations 정리·취소·원root 검증·commit을 유지한다. 단순 파일 수 동등만으로 무변경을 결정하지 않는다. 이 변경의 실제 회귀/GUI 인수는 진행 중이다. 기본 비활성 library-scan-profile-fixture는 고정9개단계숫자만 기록하며 제품 성능 판정 근거로 사용하지 않는다.

## 배경 파일 목록의 마지막 전달 관측

DeliveredState는 worker가 bounded 큐에 마지막으로 성공 전달한 actual root·행/오류·감시 건강 상태의 일시 관측이다. DB 상태나 파일 권한이 아니며 직렬화하지 않는다. 자율 재확인의 내용이 정확히 같을 때만 큐 게시와 repaint를 생략한다. 사용자 serial/root 응답은 동일 내용이어도 전달하며 stop/disconnect/실패 전송은 마지막 키를 변경하지 않는다. Snapshot·UI·마지막 키는 최대128행의 Arc 페이지를 공유한다. 실제 scan·취소·StopACK와 독립 MetricsPanel의 root revision·볼륨 표본 갱신은 유지한다.

## 일반 파일 이벤트의 재확인 합치기

OrdinaryHintGate는Download 상태가아닌WatchSession의일시관측일정이다. pending한비트와고정next_due만보유하고일반native/rootmtime힌트를초당최대1회dirty로합친다. 새로운힌트가deadline을연기하지않아연속변경과마지막힌트가남는다. attach/health/backend오류/rootidentity손실/30초보완은즉시dirty로pending을flush한다. 경로·alias·이벤트kind를권한으로해석하거나버리지않는다. 실제DB반영은기존poll/inflightscan비용을포함하며무조건1초라는보장이아니다. 수동명령은이합치기정책을거치지않는다.


## Windows 목록의 최신 파일 메타데이터

Fresh file metadata는 디렉터리의 이름/opaque ID 관측과 별도로 조회한 원 파일의 크기·수정시간이다. hard-link의 directory entry는 다른 alias를 통한 수정 뒤 오래된 값을 유지할 수 있으므로 directory EndOfFile/LastWriteTime으로 정확 동일 집합을 판정하지 않는다. 선택적 Basic-by-name query의 원volume/full128ID를 enumeration ID와 대조하고, 미지원 Windows는 같은 원 no-reparse HANDLE의 StandardInfo/BasicInfo/full128ID를 사용한다. preflight에서만 API capability를 낮추고 transaction의 오류·ID 교체·취소는 rollback한다. 이 조회도 mutable namespace/content를 고정하는 권한이 아니며, 이미 관측한 rename 후보는 게시 직전 재검증한다. 구현/최소 red→green 이후 전체·OS·비용 인수는 별도다.

## 목록 페이지 조회와 재확인

Page navigation은 마지막으로 commit한 library snapshot의 keyset으로 다음128행을 읽는 명령이다. 전체파일 관측이나 최신성 확인을 요청하지 않는다. 시작/수동Refresh/저장위치변경/WatchSession hint는 기존 전체fresh reconciliation을 유지한다. 페이지조회도 단일worker에서 수행하고 root/data 분리·cancel·bounded ACK를 유지하며 조회 실패는 DB와 원파일을 보존한다.

페이지조회는 READ_ONLY/NO_MUTEX/NOFOLLOW로 기존SQLite만 열고migration을호출하지않는다. 없는DB·legacy/손상DB·exclusiveBusy는기존파일과관측을보존하며오류를반환한다. SQLite busy_timeout5초는설정된대기정책이고단일OS호출의강제중단이나WAL/hotjournal sidecar모든변경금지를뜻하지않는다.

### 감시 root 관측

WatchSession은 active poll의 폴더 identity와 mtime를 하나의 Metadata snapshot에서 관측한다. attach 전후 identity 비교와 오류 시 detach는 유지한다. 이 관측은 경로 생애를 고정하거나 Windows created(None) identity를 강화하지 않는다. quiet polling은500ms 후보, scan중checkpoint/watchpoll은100ms이고ordinary1초·retry2초·보완30초는별도내부정책이다. quietwatch전달의추가지연은최대500ms+OS일정이며무조건1초DB반영보장은없다.

### 라이브러리 조회의 임시 비용

Windows full metadata 관측은 매번 수행한다. 변경 없는 snapshot에서는 rename을 위한 TEMP identity 인덱스를 만들지 않고, publication이 필요한 경우에만 만든다. persistent identity 인덱스와 path 중복 검사는 유지한다. 최신 파일 조회 Api는 bounded UTF16 scratch를 소유해 경로마다 비우고 재사용하며 파일의 metadata 자체를 캐시하지 않는다. DLL/query와 scratch 수명은 같은 owned Api 안에 있다.

## 주기 라이브러리 확인의 예산 후보

WatchCause는 Periodic < ChangeHint < Recovery 순의 고정 크기 원인이다. 30초 요청·1초 일반 hint gate·2초 재접속 정책을 유지하며 debounce 대기 hint가 주기 요청과 겹치면 빠른 변경 확인을 우선한다. Periodic에만 core의2ms active/500ms slot/100ms 이하 대기·180초 협조 기한 후보를 연결했다. 실제 측정과 인수 전 최종 채택으로 표시하지 않는다. 개별 OS/SQLite 호출은 기한을 넘길 수 있고 완료 지연은30초 보장이 아니다.

Worker는 대기 중 명령 한 건과 원인 한 건을 보관한다. Next/우선 hint는 원 미게시 scan을 rollback한 후 처리하며 command serial ACK를 유지한다. 예상 선점은 사용자 오류로 게시하지 않는다. Stop은 rollback/소유 자원 drop 뒤 ACK한다. 실제 실패·기한은 기존 게시 DB 목록을 보존하고 오류를 알린다. 추가 thread/mutex/이벤트 배열은 없다. 실제 동일 library.sqlite TEMP 단계의 독립 read/write·cancel/reopen은 Windows 합성 fixture로 통과했지만 main publication lock과 GUI CPU/CloseWait 인수는 별도다.

### 원 worker CPU 관측 예산 후보

Windows 주기 확인에는 원 worker thread ID·creation FILETIME을 고정한 cumulative kernel+user100ns counter를 추가했다. core CpuBudget은 최초 표본 이후 누적 CPU를 wall 시간의0.4%와 대조하고 부채가 있으면100ms 이하로 기다린다. sampling도100ms 간격이며 Store 열기 비용부터 포함한다. 2ms/500ms wall pacing·180초 협조 기한·명령/실제변경/복구/Stop 우선순위를 함께 유지한다. counter 실패·역행은 원 transaction rollback 뒤 고정 오류이며 기대 선점으로 숨기지 않는다. 추가 thread·mutex·의존성·schema는 없다. 누적 여유가 생길 수 있어 엄격한10초 sliding quota나 단일 호출 hard quota를 뜻하지 않는다. 실제 GUI·완료 지연 검증 전 후보이며 다른 OS는 CPU counter 미지원 wall-only 후속이다.

2026-10-06 단독actualclock/channel/WatchSession 비교에서1500µs는180.008초Cancelled/0.671875CPU초였고2ms는146.459초완료/0.578125CPU초였다. 더많은대기/표본조회비용을줄인2ms로후보를조정하며0.4%와180초·원GUI기준을유지한다. 제품native완료/전체GUI인수는새소스에서별도로검증한다. CPUeligible전wall창소모에대한별도가상반례초안은실제수정으로채택하지않았으며후속가설로남긴다.

## 동일 프레임 raster 재사용 실험

CPU native host는egui 상태·접근성·플랫폼 처리와전체present를계속한다. 원surface age1, 크기/scale/정확Mesh·clip동일, texture set/free 없음인 경우만이전XRGB픽셀을재사용할후보로삼는다. 이전primitive의소유Vec는Frame struct와실제capacity합산2MiB상한이며추가pixel canvas는없다. hidden/zero-size/resize/texture변경/present실패는무효화한다. 현재실험통합이며Windowsstrict10초CPU1%·80MiB·실제pixel변경/복귀인수전으로최종스택채택을뜻하지않는다.

## 목록 worker의 Stop wake 후보

StopWake는새사용자명령이나persisted상태가아닌기존capacity1목록명령채널의내부wake다. Stop bit를Release로먼저설정한뒤try_send하며full이면기존명령이receiver를깨운다. Worker는receive직후Stop을확인하고원owned자원drop후ACK한다. wake-only는DB스캔/저장위치선택/사용자serial ACK를만들지않는다. quiet대기를500ms로늘리는실험의Stop관측지연을피하며inflight100ms중단check는유지한다. 새채널/thread/mutex는없고현재실험인수중이다.

## 주기 예산의 CPU eligibility 조합 후보

Budget::at_with_cpu는CPUwait가남아있으면새wall active창을먼저열지않는다. 전체180초기한은CPUwait보다우선하고대기조각100ms를유지한다. CPUeligible일때만actualnow에서기존2ms active/500ms slot을시작한다. 누적CPU4/1000·no catchup·Stop/command/higher-watch 우선·실패rollback은변경하지않는다. 현재순수회귀/native인수전이며실제180초실패의유일원인으로확정하거나최종스택채택으로표현하지않는다.


### 2026-10-06 스캔 중 감시 조회 cadence 후보 구현

현재 구현은 scan watch.poll을 SCAN_WATCH_POLL=500ms로 제한한다. quiet 대기는 기존 500ms이며 Stop/명령 확인·CPU 샘플·대기 조각 100ms, wall 2ms/500ms, 누적 CPU 4/1000, 전체 180초와 감시 내부 30/1/2초 정책은 유지한다. native 이벤트와 root identity 확인을 생략하지 않는다. 감시 인식은 500ms+checkpoint/OS 지연이 가능하며 DB 반영의 hard deadline을 뜻하지 않는다. 동일 단독 바이너리 C1 137.071초/CPU 0.546875초와 A2 151.399초/CPU 0.593750초가 모두 완료했지만 이전 A1/B1과 실제 GUI 완료 실패를 보존한다. 실제 제품 완료·Next·Stop·root 복구·GUI CPU 검증은 진행 중이고 스택의 Proposed 상태를 채택 완료로 바꾸지 않는다. 추가 의존성·thread·schema·Cargo 변경은 없다.

## 앱 설치 기반의 검증 경계

Bundle manifest는 앱 파일의 bounded 설명이며 공급자 trust가 아니다. Payload observation은 원 열린 파일의 실제 hash·크기·PE64 구조·native identity를 관측한 자료로 이후 변경을 막거나 실행·삭제 권한을 부여하지 않는다. Fresh installation root는 원 생성 HANDLE과 private ACL·조상/root identity를 보유한 새 Windows 디렉터리로 기존 경로를 채택하지 않는다. 관리 외부 도구와 별도 권한을 사용한다. 아직 publication/active pointer/upgrade/remove 상태는 제공하지 않으며 실제 installer 인수 전이다. [제안과 구현 범위](docs/adr/0005-app-install-foundation.md).

후속 PreparedBundle은 원 설치 root/child handle을 보유한 새 로컬 copy의 준비 상태다. PreparedRecord는 정확 파일 name/size/SHA/nativeID와 root anchor의 bounded 자료이며 signed trust나 active installation이 아니다. PrepareFailure의 target hint는 일부 생성 여부가 불확실할 수 있는 진단 자료이고 삭제 권한이 아니다. 원 partial objects는 자동 정리하지 않는다. [상태 범위](docs/work-items/S07-windows-app-prepare-transaction.md).

PreparedInspection: 사용자가 명시한 prepared 폴더를 읽기 전용 원 HANDLE로 다시 검사하는 관측 결과. 원 root/file native ID, private ACL, bounded marker, exact namespace와 payload SHA를 비교하며 설치 등록·신뢰·삭제 권한으로 승격하지 않는다. source552 Windows 전체4검사·실제31회귀 및 별도 이전 copied CPU fresh inspection을 통과했다. 실제 설치 등록·제거는 후속이다.

Local development install CLI는 명시한 로컬 development_only manifest에서 prepare/inspect/prepare-register/inspect-registered 및 외부 receipt 생성/두 경로 readonly 대조/명시 제거를 제공하는 Windows 보조 명령이다. inspect/startup은 삭제하지 않으며 remove-registered에만 별도 factory를 연결한다. 현재 등록·제거는 consistency와 원 capabilities의 범위이며 active installation·publisher trust·역사 ownership·upgrade/updater 완료가 아니다.

Fresh discard는 같은 실행이 생성한 PreparedBundle의 원 HANDLE을 consume하는 명시적 비원자 정리다. DiscardFailure는 아직 disposition을 승인하지 않은 원 capability와 immutable record를 보유하며 retry는 이 동일 remainder만 사용한다. accepted disposition은 close 후 namespace 제거 가능성을 뜻하고 secure erasure가 아니다. 재시작 후 marker나 경로는 삭제 권한이 아니다. unknown 파일은 보존하고 marker·빈 root를 마지막으로 처리한다. source555에서 실제 fresh discard 인수를 통과했으며 cold uninstall은 별도다.

FreshRegistration은 원 PreparedBundle과 이번 실행이 새로 만든 registration record/seal의 원 HANDLE을 결합한 개발 상태다. schema/domain/nonce/native IDs 및 원 prepared canonical SHA를 상호 검증한다. RegisteredDevelopmentOnly는 active installation·publisher trust·재시작 후 삭제 권한을 뜻하지 않는다. RegistrationFailure는 원 준비 묶음과 새 registration capabilities를 보존하며 Drop에서 삭제하지 않는다. source558 fresh 등록 인수를 통과했으며 readonly 재시작 검사는 후속이다.

RegisteredInspection은 새프로세스의ReadRoot/ReadFile cap에 원등록record/seal·ready·payload ID와canonicalSHA/privateACL/exactnamespace를검증한readonly결합이다. source560 actual재검증을통과했으며 권한확인이나coldDELETEcap로승격하지않는다. callerexecutable cursor는verify_current에서보존한다.


2026-10-06 개발 CLI 네 명령 prepare/inspect/prepare-register/inspect-registered 연결을 source560 전체4검사·설치53/parser3회귀·실제16명령과 별도원nativeIDs·현재legacy제어로 검증했다. 등록 inspection은readonly이고 active설치/삭제권한/서명trust로승격하지않는다. 다음 durable 외부authority anchor와 cold removal은 후속이며 ADR 전체 Proposed 상태를 유지한다.

External registration receipt: 원 FreshRegistration에서새외부private루트에생성한bounded consistency사본. 원 receipt root/file IDs와nestedrecord/seal을결합하며scope검증만제공한다. source563 전체4검사/설치61회귀/실제5cases/원native2IDs 검증완료. 재시작읽기대조와삭제authority는별도이며같은SID위조방어/역사적소유권이아니다.

ReceiptInspection은원readonly root/filecap에서외부receipt의canonical64KiB/nestedtypedbinding/privateACL/정확1namespace/원2IDs를대조한다. source565 전체4검사/설치65회귀·원source563 receipt 별도process/4typedfailure 보존PASS. linked등록현재파일/삭제권한/역사소유권검증은아니며후속두경로대조로분리한다.

RegisteredPairInspection은사용자가명시한두경로의원registered/receipt readonlycaps를동시에보유하고원전체canonicalrecord+seal을대조한다. source567 전체4검사/설치68회귀/별도source563원producer 대조/4typedfailure PASS. 각각유효한다른receipt혼합은실제nativeunit에서InvalidData로거절하며copiedfreshfixture에는그검증을소급하지않는다. 순차관측이며atomic snapshot/역사ownership/삭제authority는아니다.

RegisteredRemoval은 별도 명시 요청 factory에서 원 registered/receipt 대조를 consume하고 immutable expected IDs와 원 ReadRoot를 유지한 채 original-root-relative DELETE caps를 취득한다. disposition 전 full preflight, unknown 보존, 원HANDLE payload→metadata→빈 root, failure의 원remainingcaps retry를 제공한다. source570 Windows 전체4검사/설치75/실제4묶음·7행/외부receipt4 nativeID 대조 PASS. readonly inspection과 persisted JSON은 자동 삭제·historical ownership 권한이 아니며 CLI 연결과 active/upgrade/clean OS 인수는 별도다. 최대64개 묶음의 repeated SHA는 O(N²), 약16GiB 추가 읽기 가능성이 있으며 hard OS deadline/secure erasure 보장은 없다. 신규dependency/lock 변경 없음.

현재 일곱 명령 개발 CLI의 외부receipt 생성→별도 두경로readonly 대조→명시 정확묶음 제거를 source570 Windows 전체4검사/설치75/parser4/실제28명령/현재복사중제어2/native18ID root·독립대조로 인수했다. 명시 remove 이외 inspect/startup은 삭제하지 않는다. stdout phase·mutation·accepted dispositions·pre-dropcap관측과 afterexitcap없음을 구별한다. 일반activeinstaller·upgrade/역사ownership/서명trust 게이트는 남고 ADR 전체 Proposed는 유지한다.

ExplicitActiveSelection은 caller가명시한이미검증된등록+receipt pair를새private root의boundedcanonical marker에선택하는개발용 consistency후보다. 재시작은active/등록/receipt세경로를모두명시해원ID/SHA/namespace를readonly대조한다. 저장path에서자동탐색/launch/삭제authority를만들지않으며 기존active교체/upgrade/journalrepair는후속이다. 원root/FileIDs와domain SHA·selectedexe를결합하고 같은SID위조·historicalownership/publishertrust를보장하지않는다. Windows synthetic 선택6cases·별도process readonly대조 인수는완료했으며 정식installer/upgrade는후속이다. ADR Proposed를유지한다. 신규dependency/lock변경없음.


OwnedSelectedProcess는 명시3경로 readonly선택검사의 원caps와exe/ancestor핀을보유한Windows실행후보다. 원child를suspended생성하고원Job에bind한뒤childimage nativeID/SHA를resume전에대조한다. 종료는Natural/Cancelled/TimedOut와terminal_unconfirmed를구별하며resume_attempted/may_have_executed를성공resumed와구분한다. Windows 실제3case의원process 실행·종료 인수는완료했으며 실제GUI/정식installer는후속이다. ADR0005 Proposed를유지한다.

## 개발 선택 실행의 startup 기한과 감독 수명

`run-selected`는 사용자가 명시한 active·등록·외부 receipt 세 경로를 검사한 원 capability에서 선택 앱을 감독 실행하는 개발 명령이다. `--startup-timeout-seconds`는 기본300초, 1–600초이며 검사·생성·원 child image 검증·Resume까지 적용한다. 시작 후 `wait_cancelled`는 runtime deadline 없이 자연 종료 또는 취소를 기다리고 원 Job·exe·선택 pins를 supervisor 수명 동안 유지한다. nominal100ms poll은 OS 호출·스케줄링·정리 시간을 포함한 물리 취소 지연 보장이 아니다. supervisor 종료/Drop은 terminal 확인이 아니며 영구 detach·정식 installer를 제공하지 않는다.

새 MSVC target 전체 fmt/clippy/test/release는 완료했다. 테스트 집계760 PASS/0 FAIL/13 ignored이며 source576은 이전 같은 파일 수의 snapshot과 다른 정확 hash map이다. CPU c9f09dc0…8,709,632B, CLI ab8a6cde…795,136B다. 이전 MSVC CLI786,944B 대비8,192B 증가했고 CPU 크기는 같다. 신규dependency/lock 변경은 없으며 같은 CPU 크기가 동일 binary SHA나 과거GUI 비용 검증을 뜻하지 않는다. 실제 시작2초 기한을 넘긴 실행4초 이상 유지·자연 종료·취소·밝은 테마 재시작을 root/독립 검토했다. 새 CLI 실제17명령/14사례·원nativeID10·원파일 보존도 통과했다. 기존 bounded 실행의 새 바이너리 회귀는 실제15명령/13사례·GUI4까지 root/독립 PASS다. 부모 종료 후 자식만 살아 있는 분기는 새 source에서 별도 검증 중이다. [현재 전체검사](docs/validation/evidence/windows-msvc-run-selected-corrected-workspace-20261006-continue/result.json). 정식 설치·upgrade·서명·publisher/legal/cleanOS와 P0 전체 완료는 미주장이고 ADR0005는 Proposed를 유지한다.

원 selected Job의 active_process_count는 retained process creation과 원 Job HANDLE에서 읽은 순간 생존 수다. 완료·PID 기반 소유권·미래 생존 보장이 아니며 종료 정책은 원 leader signal과 Job empty 확인을 함께 유지한다. descendant-only synthetic fixture에서 관측하지만 제품 CLI poll 경로에는 추가하지 않는다. [후속 실제 인수](docs/work-items/S07-windows-selected-descendant-lifetime.md).

## 새 개발 설치 묶음의 원 capability

FreshUserBundle은 명시한 새 outer root와 registered/receipt/active 고정 자식을 원 생성 HANDLE로 유지하는 개발 coordinator다. source preflight 이후 outer를 생성하고 등록·외부 receipt 대조 이후 active를 마지막으로 게시한다. 기존 경로 채택·교체·Drop 삭제·durable journal 회수를 제공하지 않는다. 실패의 stage와 mutation 가능성은 capability 보유와 별도이며 preflight 실패에는 원cap이 없을 수 있다. UserBundleFailure의 boxed private 상태는 실제 보유 원cap만 보고한다.

Fresh readonly observer는 원 FreshRoot에서 LIST/ATTR/READ_CONTROL/SYNCHRONIZE 부분집합 HANDLE을 복제하고 원root-relative 파일의 immutable expectedNativeID를 대조한다. 쓰기·삭제·생성 권한은 없고 SHAREALL은 원 fresh writer cap과의 호환 정책이다. 일반 cold inspection의 shareREAD 정책은 유지한다. namespace·canonical·nativeID·전체SHA 대조는 순차 관측이며 역사적 소유권·publisher trust·atomic snapshot을 증명하지 않는다. 새 생성 root에는 LIST 권한을 포함하지만 observer 복제 시 WRITE/DELETE는 제외한다.

source579 corrected 전체4검사·774/0/13은 실제 root/독립 검토 완료다. 이후 capability 보고 교정의 새 전체 검증과 실제 prepare-user-bundle/재시작/복사 중 제어 인수는 진행 중이다. 신규 dependency/schema/lock 변경은 없다. 정식 installer·upgrade·rollback·clean OS·서명 및 전체 P0 완료와 구분한다.

지표·appearance worker의 StopWake는 기존 capacity1 입력 채널의 비차단 종료 통지다. atomic Stop을 먼저 설정한 뒤 try_send하며 Full은 원 selection/write를 바꾸지 않고 이미 대기 receiver를 깨울 조건이다. appearance의 graceful busy Stop은 기존 accepted-latest 저장을 먼저 drain하고, Drop의 즉시 abort는 기존 정책을 유지한다. metrics는1초 관측 기한을 기다리되 normal Stop과 terminal mailbox poison은 같은 통지를 사용한다. 정상 freshness 확인은 선택 명령/I/O를 수행하지 않는다. StopWake는 진행 중인 OS query를 강제 중단한다는 뜻이 아니다.

명시 묶음 재검사(UserBundleInspection)는 caller가 지정한 outer의 원 readonly ReadRoot와기존ActiveSelectionInspection을수명전체보유한다. 고정registered/receipt/active 세자식namespace를검사전후/verify_current전후확인한다. childmetadata의저장경로를자동탐색입력으로사용하지않으며생성·repair·삭제·실행·historicalownership 권한을부여하지않는다. 취소/기한 오류도 원files를보존한다. 새저장schema가아니며native인수진행중이다.

InstallIntent와 InstallPhaseEvidence는 실험 중인 앱 설치 journal 계약이다. 고정 namespace의 immutable intent와 연속 단계 prefix를 현재 root/file ID, 목표 경로, manifest SHA, 이전 기록 SHA와 함께 관측한다. 미완료 prefix와 Active fingerprint만으로 설치 commit을 인정하지 않는다. UserBundleInspection으로 실제 manifest와 active record를 함께 대조해 현재 일관성만 확인한다. crash 이전 capability·소유권·삭제 권한을 복원하지 않는다.

FreshJournalAwareBundle는 새 development bundle의 실제 검증 단계 callback에서만 journal을 기록한다. 성공과 실패 모두 원 bundle/journal capabilities를 보존하며, 이전 기록 bytes를 원 HANDLE로 대조한 뒤 다음 단계를 기록한다. 별도 Noop 경로는 기존 생성 동작의 추가 단계 I/O를 생략한다. 현재 성공은 원 기록의 순차 일관성이지 power loss 내구성·crash 이후 소유권·삭제/자동 복구 권한이 아니다.


## 개발 설치의 네이티브 bootstrap 후보

LocalJournalRequest는 명시한 로컬 manifest/source/bundle/journal 네 경로의 worker 요청이다. LocalJournalReport의 capability presence는 service가 반환 전 원 cap을 획득해 검증한 관측이며 반환 후 원 cap 유지·개수·historical authority를 뜻하지 않는다. UI는 단일 worker의 bounded 최종 보고와 실제 JoinHandle 종료를 확인한 뒤 성공을 표시한다. 설치 중 닫기는 cancellation 뒤 worker 종료를 기다리는 CloseWait이며 부분 파일/journal을 Drop으로 지우지 않는다. 별도 Windows 표준 폼 후보는 기존 제품 CPU renderer/stack 확정과 구별한다. 실제 private GUI/오류/close 인수는 진행 중이다.

ExplicitBundleRemoval은 별도 사용자 제거 요청으로 완전한 현재 bundle·외부 journal을 검사하고 원 readonly roots와 immutable ID/SHA에서 DELETE caps를 획득하는 구현 중 계약이다. 활성 선택 marker를 먼저 철회하고 payload·등록 metadata·receipt·빈 child roots·outer 순으로 disposition을 요청한다. 실패는 보유 원 readonly/DELETE caps와 수락 횟수·mutation 가능성을 분리한다. persisted journal은 역사적 ownership이나 자동 제거 권한을 부여하지 않으며 외부 journal/source/data/영상/도구는 제거 대상이 아니다. disposition 수락은 경로 부재·secure erasure 증명이 아니다. [작업](docs/work-items/S07-windows-explicit-user-bundle-removal.md).

개발 설치 화면의 Removal confirmation은 별도 현재 Verify 뒤 사용자가 선택하는 경로 snapshot이다. 이 상태는 삭제 권한이 아니며 확인 시 새 worker가 현재 bundle/journal을 다시 검증한다. 돌아가기·확인 중 Close에는 제거 worker를 시작하지 않는다. Removal report는 worker가 capabilities를 놓기 전 사실을 캡처한 plain DTO이며 일부 disposition 뒤 중단과 시작 전 거절을 구분한다. 완료는 실제 outer부재·4root 수락·오류 없음일 때만 표시한다. pending readonly bundle roots 수는 DeletionRoot 내부 root와 외부 journal을 제외한 논리 cap 관측으로 OS HANDLE총수가 아니다. 현재 개발 제거 UI는 Windows844회귀·실제11GUIcase와root73native명령에서 scoped 인수했다. 새 창의 기존 검사와 비용·정식installer는 별도 게이트다. [작업](docs/work-items/S07-windows-native-development-removal-ui.md).

LocalInspectionRequest는기존개발bundle과외부journal의두명시root만갖는readonlyworker요청이다. 원source/manifest입력은새설치요청에만필요하다. UI의idle검증버튼활성화는검증완료/삭제권한이아니며사용자선택전worker를시작하지않는다. 새창의현재Verify성공후에만제거확인을사용하고확인은두rootsnapshot을고정한다. 기존4path검사API도동일private검사helper를사용한다. 현재후속통합검증중이다. [작업](docs/work-items/S07-windows-native-existing-bundle-reverify.md).

- Folder picker session: 현재 UI의 한 저장 위치 draft를 고르는 transient OS dialog. busy→completion은 용량1이며 취소 요청 이후 선택값은 적용하지 않는다. 다운로드에서는 draft만 갱신하고 파일 목록에서는 기존 위치 Apply command로 검증·저장·scan한다. cancel/drop 자체는 native cleanup ACK가 아니다. 원 창 종료는 worker 완료를 기다린다. 저장된 위치의 startup 복원은 사용자의 후속 입력과 CloseWait를 덮어쓰지 않는다. [결정·검증 범위](docs/adr/0006-native-folder-picker.md).

## 미디어 작업 화면 후보의 상태 계약

Prepared media는 실제 metadata 확인을 통과한 일시적 읽기 대상이다. Download job이나 Artifact가 아니며 준비 성공은 저장·삭제·인증 권한을 부여하지 않는다. Selection은 알려진 길이 안의 `0 <= start < end <= duration` 구간이며 Playhead는 구간과 독립된 내용 확인 위치다. 수치 입력 실패는 마지막 유효 Selection을 바꾸지 않는다. 전체 선택도 Playhead를 바꾸지 않는다.

Preview generation은 입력 변경·seek·취소 때 증가한다. worker는 실행 하나, 최신 pending 하나, completion 하나만 보유한다. 오래된 콘텐츠 결과는 적용하지 않으나 OwnershipUnconfirmed는 세대와 무관하게 sticky 실패다. Thread stop ACK와 프로세스 정리 확인은 구별한다. 숨김/다운로드 진행 중에는 새 미리보기 준비를 시작하지 않는다. Source URL은 일시적인 direct media URL과 구별하며 direct URL은 영구 job 식별로 저장하지 않는다.

이 후보는 로컬 파일 읽기와 단일 직접 미디어 URL의 저해상도 무음 미리보기 범위다. DRM/live/playlist/별도 audio-video/특수 인증 header가 필요한 소스는 지원 완료로 보고하지 않는다. UI clock의 nominal 위치는 독립 검증된 PTS가 아니다. 원격 전체 수신 byte 및 decoder RSS의 hard cap도 현재 확보되지 않았다. [작업 기록](docs/work-items/S00-media-workspace-interaction.md)의 남은 인수를 따른다.

Windows 복귀 listener의 stop event는 server 소유 anonymous manual-reset event이며 원 pipe connect 완료와 함께 기다리는 wake capability다. 취소 Atomic을 대체하는 종료 성공 증거나 외부 프로세스/파일 삭제 authority가 아니다. 원 pending I/O drain과 token identity 검사를 유지한다.

관리 도구의 공급(설치)과 활성화/사용은 구별한다. Archive 공급은 inactive, yt-dlp의 기존 direct 공급은 검증 후 active다. 앱 재실행 복원은 기존 active+빈 field에만 적용하며 사용자 경로를 덮어쓰지 않는다. 선택 도구의 공급 결과와 app bundle 성공은 분리된 상태다. 도구실패를app 설치실패/삭제로바꾸지않는다. suite absolute deadline은 각tool의새120초시작으로연장하지않는다.

## 로컬 구간 저장 후보

Local range export는 원 Prepared pin과 유효 Selection에서 새 사용자 결과를 게시하는 단일 요청이다. 최신 세대 미리보기와 달리 취소·세대 변경으로 실제 생성 사실을 버리지 않는다. ExportCompletion의 파일 결과와 cleanup_uncertain는 독립이다. 게시된 파일을 검증했더라도 관리 scope 종료가 미확인이면 새 작업·정상 종료를 차단한다. worker stop ACK는 프로세스 정리 성공이 아니다. 실패의 fresh_directory/stage_preserved/final_created는 경로 사실이며 삭제 권한이 아니다. 개발 후보는 원 create-only stage와 최종 clip의 hardlink를 보존하며 원본·기존 출력·partial을 자동 삭제하지 않는다. [구현·미검증 범위](docs/work-items/S05-local-range-export-20261011.md).

## 도구 경로 기억과 현재 실행 선택

Tool preference는 yt-dlp·FFmpeg·Deno 각각의 Automatic 또는 Manual(절대 경로) 선택이다. Current tool path는 이번 실행에서 입력하거나 검증된 managed 활성 경로이며 preference와 다를 수 있다. 시작 환경값은 존재 자체(빈값 포함)가 저장값보다 우선한다. 직접 경로 편집은 빈값도 Manual 의도로 기록하며 늦은 조회·저장 결과가 그 편집을 덮어쓰지 않는다. Preference saved는 최신 요청과 현재 draft가 같은 실제 저장 완료만 의미한다. 이전 snapshot만 완료된 경우 현재 편집을 저장했다고 표시하지 않는다.

Automatic은 현재 GUI에서 앱이 검증하여 선택한 managed 경로 또는 빈 선택이다. yt-dlp GUI의 PATH 탐색 연결은 아직 미구현이므로 모든 도구의 자동 발견을 뜻하지 않는다. 도구 경로 저장은 실행 신뢰·파일 소유권·삭제 권한을 부여하지 않는다. 종료는 unsent 변경을 폐기하고 이미 active인 설정 worker의 ACK를 기다린다. [진행 중 인수](docs/work-items/S04-manual-tool-preferences-20261011.md).

## 선택 기록의 새 이름 공개

Windows 개발용 ExplicitActiveSelection은 canonical record를 새 private root의 active-selection.pending에 쓰고 원 bytes·cap/등록 pair를 검증한 뒤 원 file/parent HANDLE의 non-replacing NT rename으로 active-selection.json을 공개한다. marker native ID와 기존 schema1은 유지한다. FreshNamePublishError는 공개 전 실패, native call 불확실, 공개 후 검증 불확실을 구분한다. 뒤 두 오류는 namespace rollback·삭제 권한이 아니며 원 root/file/pair capability와 mutation 가능성을 보존한다. 전원 손실 내구성·기존 active 교체·반복 upgrade/rollback 완료는 아니다. [작업과 인수](docs/work-items/S07-windows-fresh-publication-adapter-20261012.md).

## Windows 새 설치 루트의 정확한 생성 충돌

Exclusive collision은 원 NT FILE_CREATE 호출의 STATUS_OBJECT_NAME_COLLISION으로 확인한 기존 루트 거절이다. bundle 준비는 이 증거에서만 변경 가능성을 false로 보고하며, ErrorKind::AlreadyExists·HANDLE 없음만으로 이를 추측하지 않는다. 생성 후 ACL/identity 오류와 자식 생성 실패는 보수적 변경 가능성을 유지한다. 이는 bundle namespace에 대한 실패 분류이며 외부 경로 삭제·기존 데이터 채택 권한이 아니다. 현재 Windows adapter133회귀가 통과했고 최신 전체와 CLI 후속 대조는 [작업](docs/work-items/S07-existing-root-refusal-20261012.md)에 남는다.

## 진단 경로 선택 초안 — 통합 검증 중

Diagnostic path draft는 보고서 저장 또는 MPD 첨부 대화상자에서 선택한 일시적 경로다. 선택 자체는 파일 생성·덮어쓰기·첨부/읽기 권한을 만들지 않고 기존 명시 저장/첨부 worker가 현재 경로와 제한을 다시 확인한다. SaveDialog의 overwrite 안내가 기존 create-only export를 변경하지 않는다. transient picker의 cancel/stop 이후 완료는 초안에 적용하지 않으며, CloseWait는 기존 보고서 worker와 native cleanup completion을 모두 기다린다. 새 persisted schema가 아니며 실제 인수는 [작업](docs/work-items/S00-diagnostic-file-dialogs-20261012.md)에 따른다.

진단 결과의 표시 문맥은 일시적인 UI 정보다. 저장·MPD 첨부·로그 편집 결과를 발생한 조작그룹에서 표시하며 Job/Attempt 상태나 파일 생성 권한을 변경하지 않는다. 원 worker 결과와 취소·정리 ACK 정책은 유지한다. [현재 시각 검증](docs/work-items/S00-diagnostic-save-feedback-20261012.md)은 전체 UI/UX 완료와 별개다.

## Windows 네트워크 관측 선택과 고급 입력 draft

네트워크 관측의 자동·수동·끄기는 다운로드 라우팅 정책이 아닌 세션 관측 선택이다. 자동은 OS 기본 route 후보에서 IPv4를 우선하고 없으면 IPv6를 사용한다. 동일 family는 route metric과 interface metric의 합으로 선택한다. 다른 앱 트래픽을 포함하며 split tunnel이나 목적지별 route 때문에 실제 다운로드 연결과 다를 수 있다. 선택/route epoch가 바뀌거나 실패·끄기이면 이전 rate baseline을 재사용하지 않는다. UI/실제 Windows 통합 인수는 진행 중이다.

속도 입력 draft는 표시 단위와 편집 중 문자열이며 core 옵션은 정수 B/s다. SI 단위를 정확히 환산하고 무효 draft는 마지막 유효 core 값을 유지하되 새 다운로드를 차단한다. 단위 변경 자체는 옵션 snapshot을 변경하지 않는다.

읽기 전용 실행 계획은 유효한 옵션 draft를 소비하고 도구 선택 여부와 독립된다. 미리보기는 작업 시작 권한이 아니며 실제 실행은 기존 절대 도구 경로·준비·닫기 gate를 별도로 통과해야 한다. 무효 draft와 원 URL 전체 생략 정책을 유지한다.


## 설치 위치 추천값

설치 위치 추천값은 사용자별 기본 로컬 경로의 읽기 전용 startup hint다. 기본 위치에 일반 폴더가 이미 있으면 그 경로를 표시하고 별도 presence worker가 새 설치를 거절하며 기존 설치 확인을 안내한다. 자동으로 dlpad-1 등 다른 위치를 골라 중복 설치를 유도하지 않는다. 기존 폴더를 설치 소유권으로 채택하지 않고 외부 입력은 우선한다. 이 값은 FreshRoot·실행·삭제 권한이 아니며 실제 설치/확인은 worker가 다시 검증한다. 파일·reparse·오류 경로는 추천값으로 자동 채택하지 않는다. 이전16후보 검색은 과거 구현이다. [작업](docs/work-items/S11-default-install-location-20261013.md).

Callback repaint generation은 단일 ROOT Context의 completed pass 번호로 묶인 일시 요청이다. host는 각 generation의 earliest deadline을 두 고정 슬롯에 보존하고 소비 시 current 또는 current-1만 선택한다. 오래된 immediate deadline을 새 요청에 다시 붙이지 않는다. Context pass 조회는 queue mutex를 잡기 전에 수행하며 callback은 Context를 조회하지 않는다. 정상 settling·worker wake와 Root output·metrics freshness는 유지한다. persisted 작업 상태나 파일 권한이 아니며 실제 통합·비용 회귀는 검증 중이다.

## 관측 준비 표시와 진단 worker snapshot

관측 준비는 새 도메인 상태 enum이 아니라 Windows 수신·송신의 초기 대기 표시다. 자동·수동 선택의 첫 요청이 pending이거나 이미 worker에 전달되었어도, 아직 latest 표본이 없고 failed/Stop/종료 guard가 없는 동안은 `수신·송신 관측 준비 중`으로 표시한다. fresh Observing/no-rate의 건강한 baseline 준비도 같은 문구를 사용한다. cached stale 표본·관측 실패·Stop·Off·NoRoute의 기존 분류를 정상 rate나 완료 상태로 승격하지 않는다. 첫 명령 dispatch 뒤 pending=false라는 이유만으로 초기 None을 측정 불가로 표시하지 않으며, SQLite·선택 revision·worker 완료 권한의 새 상태가 아니다.

Worker snapshot은 default-OFF first-paint-fixture owner가 기존 main-frame publication에 덧붙이는 상한 있는 숫자 관측이다. worker operation counter와 epoch, 마지막 main publish의 frame ordinal, weak snapshot/overflow를 구별한다. frame lag는 시간이나 TID별CPU 귀속이 아니며 마지막 publish 이후의 worker 작업은 아직 보이지 않을 수 있다. 계측 빌드 성공/진단 비용 성공을 기본 앱 성능 또는 전체 P0의 성공으로 확대하지 않는다.


## Windows 창별 제목 표시줄 테마 요청

제목 표시줄 테마 요청은 CPU host의 retained Window에서 얻은 현재 owned HWND에 한정한 UI thread의 표시 요청이다. 초기 실제 egui frame와 원하는 tuple 변화에서만 시도하고 동일 frame 반복·unsupported HRESULT에는 매번 재시도하지 않는다. DWM 20번은 resolved dark를 따른 opt-in이며, 수동 Light/Dark는35/36번 caption/text COLORREF를 앱 panel_fill/text_color에서 요청하고 System은두색을COLOR_DEFAULT로되돌리도록요청한다. 세 setter의HRESULT는독립적이며production paint는반환값을보관하지않는다. 시스템테마·전역설정·별도window hooks·새실행/파일소유권정책은추가하지않는다.

속성 요청의 source 구현, DwmGet readback, private raster와physical DWM 인수는각각다른증거다. 현재f15 CPU의Light→Dark→System private Get20은0/1/0이고35/36 Get은E_INVALIDARG로미검증이며Set HRESULT도미관측이다. 따라서manual caption/text 적용이나System기본색복귀가검증됐다고표현하지않는다. privateclassic OSchrome는지원/물리표시의확정판정이아니며전체GUI·사용자인수를뜻하지않는다. [작업 계약](docs/work-items/S00-native-titlebar-theme-consistency-20261013.md).


네트워크 수동 이름의 syntax-invalid 설명은 worker 관측 실패와 별도다. 기존 문법은 빈 이름을 허용하지 않고 UTF16 256단위 이내이며 제어 문자가 없어야 한다. 현재 길이 초과 오류 설명의 압축 후보는 이 거절 조건을 바꾸지 않는다. 오류 문구의 시각 압축은 이 문법·선택·마지막 유효 관측·worker 정책을 바꾸지 않는다. 직전 f15의800Dark200 label3 시험은 라벨·간격·입력·오류 합236px가본문219px를넘어실패했고, 짧은 설명 후속수정은 이후0093 CPU에서 실제 Dark200/Light100 label3 시험으로 검증했다. 기존 문법·worker 권한은 유지한다. 전체 코드 검사1284 PASS를이문맥공개성공으로사용하지않는다.

## 설치 후 실행 요청과 supervisor 수명

설치 UI의 verified는 버튼 표시 조건이다. 실행 권한은 worker가 bundle·journal·active selection을 다시 검사한 뒤 얻는 typed capability에 있다. Started는 원 process ID와 creation의 resume ACK이며 GUI 준비 신호가 아니다. 대기 중 닫기는 취소한다. Started를 수락한 뒤 정상적으로 설치 창을 닫으면 HWND만 제거하고, 원 설치 프로세스와 Job supervisor는 앱이 자연 종료할 때까지 유지한다. 비정상 message loop 종료나 owner 포기는 취소한다.

FailedCompletion은 원 exit code와 실행 가능성·종료 확인 여부를 마지막 receipt에 보존한다. 정리가 확인되지 않으면 원 capability를 유지하고 수정·새 실행을 막는다. 확인된 실패는 명시적으로 수락한 새 실행에서만 초기화한다. 성공 종료와 취소·시간 초과·nonzero 종료는 구별한다. v11의 실제 Windows 설치·실행 수명 및 exe 변조 거부 범위는 인수했다. 늦은 실패 오류창 재표시·감독 프로세스 비용·물리 GUI 준비·정식 배포는 별도다. [수명 결정](docs/adr/0009-installer-launch-supervision.md).

설치 완료 제목과 다음 행동 안내는 기존 verified 상태의 표시이며 새 도메인 상태나 실행 권한이 아니다. 경로 편집·실패 시 준비 안내로 돌아가고, 시작이 수락되면 설치 창이 닫힌다. [현재 UI 인수](docs/work-items/S00-installer-completion-copy-20261013.md).


IME 취소 transaction의 확정 비교 기준은 원 선택 방향이 아니라 현재 preview range이다. 성공 rollback은 원 문자열·directed cursor와 원 commit anchor를 함께 복원한다. 외부 text 변경 또는 composition anchor 밖으로 이동한 selection은 stale nonempty Commit을 계속 거부한다. 이는 새 persisted 상태가 아니며 [좁은 결정](docs/adr/0001-native-ui.md)과 [실제 회귀](docs/work-items/S00-settings-ime-positive-commit-20261013.md) 범위를 따른다.


## 관리 도구 검사와 자동 선택 요청 ticket

관리 도구의 Checking/Available/Unavailable은 worker가 관측한 임시 UI cache다. Available은 경로 문법만으로 만들지 않고 호환·해시·실제 버전·native identity 검사 결과에서 받는다. cache는 실행 권한·publisher 인증·파일 삭제 권한이 아니며 작업 준비 시 다시 검증한다. metadata inventory와 활성 DB 선택은 실제 실행 가능성과 구분한다.

ManagedAutoTicket은 Controller의 한 슬롯 path/mode revision을 담은 opaque 현재-session 요청이다. dispatch에서 생성한 ticket을 worker가 그대로 반환하며 늦은 Manual/경로 편집·환경 override(빈값 포함)·CloseWait·revision 포화는 자동 경로 반영을 거절한다. 다른 슬롯 편집은 독립적이며 background 반영은 저장을 만들지 않는다. install-only는 기존 active를 유지하고 명시 verified selection은 commit 이전 cancel checkpoint를 거친다. [구현 결정](docs/adr/0008-windows-optional-tool-install.md)의 scoped 검증과 전체 제품 인수를 구분한다.

## 파일 목록의 OS 열기 요청과 도구 표시 캐시

파일 목록의 더블클릭/Enter는 기존 내부 미리보기 gate를 따른다. 시스템 파일/폴더 열기는 별도 명시적 행동이고 worker가 선택 원본의 root·full native ID·size·mtime와 경로 경계를 다시 확인한다. HandedOff는 Windows Shell에 요청 전달을 확인했다는 뜻이며 플레이어 준비/재생 성공이 아니다. 취소/시간 초과 이후 실제 Shell 호출이 성공한 경우 이를 취소 실패로 숨기거나 사용자의 player를 종료하지 않는다. private 자동 시험에서는 Shell 제공자를 대체하고 사용자 desktop을 호출하지 않는다.

도구 표시 캐시는 세 도구의 마지막 목록을 즉시 그리는 읽기 전용 정보다. 갱신 실패도 기존 목록을 보존하지만 이 정보로 실행·선택·삭제를 승인하지 않는다. 단일 worker의 outer tool/request sequence와 업데이트 inner tool/generation이 모두 현재 pending에 맞아야 결과를 적용한다. 읽기 전용 탭 이동은 진행 작업의 origin을 바꾸지 않는다. 최신 버전 조회의 CapturedPin은 worker 메모리에서 관측한 exact release이고 설치는 그 pin을 한 번 소비한다. JSON 영수증이나 UI 표시의 버전 문자열로 복원한 설치 권한이 아니다. 새 버전 설치는 inactive로 공개하며 현재 다운로드 도구 변경은 별도의 재검증된 선택을 요구한다. GUI 현재 버전 표시는 실제 현재 입력 경로와 관측 cache를 비교하며 빈 경로는 Unknown, Manual/env/mismatch는 보수적으로 External로 표시한다.

이 계약은 v21 개발 소스에 통합되었다. 2026-10-14 Windows MSVC fmt·strict clippy·전체 test·release가 실제1452 PASS/0 FAIL/38 ignored로 통과했고 Root 원로그/3실행파일/소스664를 독립 대조했다. 앱63화면의 직접 팀 검토에서는200% 테마 선택란 상단 잘림과 재시작 초기 검은 가로 영역을 후속으로 남겼다. 정상 설치 최초 재검사는 Verify203 준비 전 요청에서 실패했고 원자료를 보존한다. 실제203/default203을 관측한 새 시험은 설치·검증·앱 실행의 자연0/Job0를 Root가 대조했다. 기존 위치 Install 비활성 정책에 맞춘 소유1바이트 변조 시험도 자연installer1/원Job총process1·0active와 원payload 보존을 확인했다. 전체 UI/설치 인수는 완료가 아니다. Windows Shell 실제 사용자 기본앱 인수와 다른 OS 인수는 남아 있다.

## 제목 파일명과 복사 내보내기의 격리 후보

FinalNameStyle은 최종 artifact의 이름 정책이며 staging/외부 argv나 다운로드 완료 증명을 바꾸지 않는다. 현재 후보의 TitleWithJobId는 제목과 full job ID를 유지하고 missing/null은 legacy다. StoredOptions version1의 optional field, engine resume snapshot version2, publication version3 title binding은 서로 다른 저장 계약이다. 재개는 원 정책과 원 argv를 함께 검증하며 파일명을 보고 정책을 복원하지 않는다.

Created-relative-file capability는 기존 부모 아래 방금 FILE_CREATE한 원 HANDLE과 identity를 갖는 런타임 소유권이다. path/JSON/기존 File로 복원하지 않는다. 공개 후 Unconfirmed의 경로·identity는 사실만 전달하고 삭제 권한을 주지 않는다. Copy export는 원본을 보존하고 새 파일을 검증한 후 공개한다. 단일 Library worker의 완료 ACK 전에는 Stop을 완료로 표시하지 않으며 화면을 옮겨도 footer 진행률을 표시한다. 완료 알림은 한 번 소비하되 자세한 결과는 파일 목록에 남는다. 이는 현재 격리 후보이며 v21 제품 인수와 구별한다. [제안 결정](docs/adr/0010-readable-artifacts-and-copy-export.md).


FileDropRegistration은 Windows 창 생성 때 관측한 세션 입력 능력이다. Disabled/Ole/NativeFallback을 구별하며 디스크 파일 identity·읽기/삭제·실행 권한이 아니다. NativeFallback은 해당 창의 WM_DROPFILES만 받아 기존 worker로 전달하며 놓기 전 hover를 보장하지 않는다. HRESULT는 내부 진단이고 사용자 안내는 중앙 catalog의 별도 문구다. OLE STGMEDIUM과 WM HDROP 정리 권한을 혼동하지 않는다. 이 계약은 격리 후보이며 v21 패키지의 현재 동작으로 보고하지 않는다. [후보 결정](docs/adr/0001-native-ui.md).


2026-10-14 후속 제품 통합: 제목 파일명·생성 상대파일 capability·복사 내보내기·네 개 상위 목적·기본 저장 위치의 사용자 serial/ACK·Windows typed Shell broker·창별 FileDropRegistration은 currentsource669에 통합했다. Windows 전체1,524/0/42·43harness와 별도nativeMSVC CI15/8Cargo/7captures를 독립 대조했다. 실제577/598설치 경로 앱 HWND/자연0와policy7/WM_DROPFILES4case를 확인했다. v21 ZIP에는 포함하지 않았으나 최신v22 ZIP에는 포함된다. userphysicalShell/IME/reader·73전체시각·18비용과정식릴리스 인수는 별도이며 제안ADR의 범위를 확대해Accepted로 쓰지 않는다. [현재 통합](docs/work-items/S00-windows-v22-candidate-integration-20261014.md).

2026-10-14 설치 revision 순수정책 실험(Proposed): revision은기존fresh bundle/active selection v1과다른generation·predecessor·정확한from/to binding의전이사실이다. Prepared/Verified/Activated는현재순수record상태만뜻하며실제디스크활성화나삭제·실행권한을만들지않는다. 같은known rollback binding만허용하고실패/취소/불명은원전이자료를보존한다. Windows순수6시험/Root검토를통과했으나durable commit/복구/bootstrap/앱데이터호환은미구현이다. [실험과 남은 계약](docs/work-items/S07-local-revision-policy-experiment-20261014.md). 기존v1namespace와잠금/schema는변경하지않았다.


## 삭제 후 packaged 재설치의 기록 선택 후보

설치기 packaged 기록 선택은 runtime UI hint이며 삭제완료 tombstone이나 과거권한 복원이 아니다. 제거한 bundle의 외부 immutable journal은 원자리 보존한다. 같은 target의 빈 설치에는 새 sibling generation을 fresh create로 요청하며, 다시 연 설치기는 bounded32 후보 중 실제 current bundle/native journal 일치가 유일한 기록을 worker에서 선택한다. 실행·제거·설치 서비스는 매번 실제 경로를 재검증한다. 명시4path 모드와 journal schema1은 유지한다. cancel/deadline/reparse/모호한 일치/후보상한은 기존자료를 보존하는 실패다. 현재 구현 검증 중이며 v24 ZIP의 동작으로 확대하지 않는다. [작업](docs/work-items/S11-reinstall-location-20261014.md).


## 현재 파일 전송 관측과 표시 후보

ProgressFeed의 current-file downloaded/total/estimated-total/status는 최신 한 개의 일시 관측이다. downloaded/확정 total은 u64이며 fragment 예상 크기는 yt-dlp의 소수 연산을 보존하는 finite nonnegative f64(u64 최대값 이하)다. 전송 중 예상값의 비율은 최대99.9%이며 확정 완료를 대신하지 않는다. 정확한 양의 total을 estimate보다 우선하며 estimate는 UI에서 추정으로 표시한다. 범위를 벗어난 양·total 미상·3초 초과는 퍼센트 근거가 없다. finished/100%는 현재 전송의 끝이며 Downloaded 상태나 검증한 Artifact를 만들지 않는다. 영상·오디오를 따로 전송하면 현재 파일 진행률이 다시 시작할 수 있다. SI 속도 B/KB/MB/GB는 기존 속도 제한과 같은1000기준이며 측정 불가를0으로 표시하지 않는다. 선택 설명은 hover/focus/고정?에서 읽고 필수 오류·인증 보존 안내는 본문에 유지한다. 현재 구현 및 통합검증 중이며 v25 앱의 동작으로 주장하지 않는다.


## 목록 검색 후보

List search query는 현재 관측 DB를 읽는 의도이며 새 scan·작업 실행·파일 권한을 만들지 않는다. Draft query와 Applied query는 아직 입력 중인 검색어와 실제 표시 중인 결과의 조건을 구분한다. Result generation은 단일 읽기 worker의 늦은 이전 결과 게시를 막는 UI 계약이며 DB revision이나 다운로드 Attempt가 아니다. 파일명·경로 및 저장된 작업 ID·URL을128행씩 전체 기록에서 조회하고, 제목이 없는 이력의 제목을 추측하지 않는다. 현재 구현·검증 중이며 최신v25 앱과 구별한다. [제안 결정](docs/adr/0011-bounded-list-search.md).

## 검색 결과의 일시 공개와 도움말 닫힘

검색의 조회 세대/게시 검색어와 UI 결과 공개 의도는 다르다. matching latest generation 최초 성공에서 현재 draft가 게시 검색어와 같을 때만 메모리 내 한 번의 scroll 의도를 둔다. 새 요청·draft 불일치·오류·Stop은 이를 소멸시킨다. Library는 빈 결과만, History는 명시 nonempty query의 첫 record/빈 안내만 드러내며 focus·selection·실행 권한을 변경하지 않는다. 초기 전체 목록·동일 세대 refresh/page는 새 의도를 만들지 않는다.

Escape로 닫은 Help의 같은 연속 focus/hover는 자동 설명을 다시 열지 않는다. 새 클릭·실제 focus/hover 재진입은 다시 허용한다. 이는 고정 logical widget별 bounded ephemeral UI 상태이며 DB/domain schema나 다운로드 상태가 아니다. 관련 수정은 Windows 새 전체·실제 화면 검증 중이고 ADR0011의 Proposed 상태를 확정으로 바꾸지 않는다.


검색 공개 helper의 bool은 현재 유효한 일회성 UI 공개 의도를 이번 호출에서 소비했음을 뜻한다. ID가 이미 보이면 true일 수 있으며 호출자는 들어갈 수 있는 주변 문맥만 별도로 드러낸다. 이는 저장 상태·focus·선택·재개 권한이 아니다. 작업 이력의 짧은 상태와 본문 제한은 기존 JobState/StoredOptions의 읽기 표시이며 새로운 상태 전이를 만들지 않는다. 현재 격리 검증 범위와 제품 통합은 [작업 기록](docs/work-items/S03-history-status-compact-20261015.md)에 구분한다.

## 영상 상세의 외부 앱 요청과 도구 진행 표시

영상 상세의 기본 앱/다른 앱 선택과 탐색기 파일 선택은 원 Prepared의 로컬 파일 pin을 보유한 명시 요청이다. URL을 로컬 파일 권한으로 해석하지 않고 worker가 원HANDLE·파일identity·상위경로를 다시 검증한다. 정확한 NoAssociation 거절에서만 같은deadline으로 선택 요청을 한 번 전환한다. OS에 요청한 성공은 실제 재생 성공이나 선택 완료가 아니다. 외부viewer/선택창은 사용자 소유이며 다운로드Job에 넣거나 앱 종료시 죽이지 않는다. Windows Explorer 위임의 실제사용자창 성공과 다른OS는 미검증이다. [통합 작업](docs/work-items/S05-media-detail-system-open-20261015.md).

도구 진행 라벨은 실제 성공 enqueue의 lookup/version-check/install/verify/remove/select를 구분하는 ephemeral 표시다. 설치 목록cache·busy·실행 재검증·worker 취소/종료ack를 대체하지 않으며 stale receipt가 새 진행 상태를 종료시키지 않는다. [작업](docs/work-items/S04-tool-background-status-20261015.md). source32에 통합했고 전체 검증을 완료했다. v30 ZIP의 동작으로 소급하지 않는다.


## P0 도구 설정 복귀와 화면 공개 후보

도구 설정 복귀 의도는 한 번의 navigation episode이며 URL/구간 draft 복사본이나 실행 권한이 아니다. 미리보기 준비 전의 로컬 retry intent는 상한 있는 Source1개뿐이며 Prepared/local 성공과 구분한다. 새 source·clear·명시 URL 변경은 이전 intent를 폐기하고 명시 retry만 기존 worker의 새 pin/도구 검증으로 이어진다. 빈 도구 입력과 최신 Available cache를 설치·실행 권한으로 삼지 않는다. 선택적 Deno를 URL만으로 필수로 판정하지 않는다.

화면 공개 의도는 새 검색 결과/새 local detail/새 outcome을 처음 드러내는 일시 bool/frame이며 사용자 focus나 이후 수동 scroll을 변경하지 않는다. History는 실제 visible viewport와 paint overscan을 구분하고, 고정된 긴 Help는 실제 owner focus·popup 열림·overflow일 때만 네 탐색 키를 소비한다. 이 후속은 v32에 통합했고 Windows 전체 검사와 100/200% private 화면의 제한된 인수를 완료했다. v31에 소급하거나 물리 입력·전체 P0 완료로 확대하지 않는다. [통합 작업](docs/work-items/S00-ui32-integration-20261016.md).


## 수동 설치 버전 전환의 검증 범위

현재 앱은 기존 설치 재검증·명시 감독 실행과 다른 폴더의 새 설치를 지원한다. 수동 side-by-side 프로필 시험은 서로 다른 개발 A/B를 같은 합성 프로필에서 A→B→A로 순차 실행해, 원 데이터와 작은 명시적 설정 변경의 보존을 확인하는 개발 인수다. 새로운 다운로드 상태나 자동 active-head 전환 권한이 아니다. v31/v32 Queued3·library2·Light→Dark→Dark와 B 설치 충돌 후 A 재실행을 실제 검증했다. 완료·실패 이력 전부 또는 미래 DB downgrade 지원으로 일반화하지 않는다. [실제 시험](docs/work-items/S07-manual-side-by-side-profile-20261016.md).

ADR0005의 durable revision/cold writer 실험은 Proposed 기반으로 보존한다. scratch Windows17회귀와 strict engine clippy 통과가 Live 제품 업그레이드 구현을 의미하지 않는다. P0의 실제 복구·업그레이드 시험은 현재 지원 정책을 대상으로 유지하며, 자동 앱 updater는 사용자 지정 기능·Git/CI 이후 후속 순서다.

기존 설치의 cold inspection 결과는 선택한 target/journal과 revision이 그대로이고 유일하게 검증된 경우에만 설치기 표시용 readiness에 재사용한다. presence만으로 실행·삭제 권한을 만들지 않으며 실행과 제거는 원 fresh 검증을 계속 수행한다. 현재 SemVer 0.1.0과 개발 검수 snapshot 번호는 다르고, 자동 앱 교체/업데이트는 별도 후속 범위다. [작업](docs/work-items/S07-installer-existing-readiness-20261016.md).


## 탐색기 선택의 Shell 경로 표기

로컬 원본 권한은 canonical 경로와 retained 원 파일/부모 HANDLE에 남는다. Windows Shell namespace 파서에 전달할 ordinary 경로 표기는 표시용 alias이며 실행 권한이 아니다. Reveal 요청에 한해 원 file/nativeID·parent/nativeID·정확한 canonical directory entry가 모두 같은 alias만 사용한다. Windows의 trailing-dot/space 등으로 다른 항목을 가리키면 거절한다. 다른 파일 열기/선택기 분기의 입력과 외부 Explorer/Job/Default desktop 계약은 유지한다. ordinary 경로를 SHParseDisplayName으로 PIDL로 바꾸고 정확히 한 번 해제한다. OS의 선택 요청 수락과 실제 선택 화면 완료는 구분한다. [원인·회귀](docs/work-items/S00-reveal-native-failure-20261017.md).


설치기 packaged 모드의 명시 재검사는 초기 admission 중단 후에도 위치·유일한 현재 journal generation 탐색을 다시 수행한다. 기본 journal 표기는 hint이며 중단 전에 current generation을 찾았다고 가정하지 않는다. worker 취소 ACK 전에는 재검사를 시작하지 않고, 빈 위치의 Vacant와 기존 설치의 검증된 readiness를 구분한다. 명시4경로 검증·실행/제거의 fresh guard·기존 자료 보존 계약은 유지한다. [재현·수정](docs/work-items/S07-installer-cancel-retry-20261017.md).


업데이트 BundleObservation은 원 StagedPackage의 native identity와 signed whole-file size/hash를 전후 대조한 뒤 현재 preview ZIP7member·manifest5payload·각CRC/SHA·binding을 읽은 bounded 관측이다. 추출 파일·설치/실행/삭제 권한이나 운영 publisher/legal 완전성이 아니다. 다음 동작은 원 StagedPackage를 유지하고 다시 검증해야 한다. [범위와 원본 검증](docs/work-items/S11-update-bundle-validation-20261018.md).


## 별도 업데이트 설치 준비

Prepared update bundle은 원 StagedPackage를 전체 재검증하고 Windows private FreshRoot에 exact7member를 전개한 뒤, readonly 원 source HANDLE과 별도 고정 FreshJournalAwareBundle을 유지한 결과다. 기존 설치/선택/프로필을 수정하거나 자동 실행하는 권한은 아니다. 부분 생성 실패는 원capability와 파일을 보존하며 경로 힌트로 정리하지 않는다. [구현과 실제 인수](docs/work-items/S11-update-bundle-preparation-20261018.md)를 따른다.
