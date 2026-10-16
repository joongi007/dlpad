# ADR-0002: 단일 앱 프로세스와 로컬 복구

상태: Proposed.

결정안: 앱 내부 supervisor, Linux 작업별 guardian/다른 OS 소유 프로세스 adapter, SQLite 작업 journal, 디스크 partial, 라이브러리 reconciliation을 사용한다. UI 창과 작업 수명을 분리하지만 기본 상시 daemon은 두지 않는다.

대안: 별도 headless daemon은 UI와 독립성이 높지만 IPC·업데이트·권한·제거 복잡성이 증가한다. 초기에는 단일 프로세스의 idle 비용과 parent-death 처리를 검증한다.

결과: UI crash가 작업에 영향을 줄 수 있으므로 journal/child cleanup/recovery는 필수. 폴더만으로 작업 이력을 복원하지 않는다. 재부팅 중 지속 다운로드는 지원하지 않는다.

## 2026-09-30 구현 증거

SQLite snapshot/revision, Attempt별 도구 증거, 실패 상태 저장, startup Interrupted 복구, bounded tail pipes와 Unix 취소를 구현했다. Publication plan을 공개 전에 저장하고 Unix hardlink identity로 재개를 증명하는 synthetic crash-gap 테스트를 추가했다. Instance guard는 같은 data의 중복 실행을 거절한다.

전체 architecture는 Proposed를 유지한다. 당시 Unix process group과 직접 child PDEATHSIG만으로 앱 crash 시 모든 자손을 보장할 수 없었다. Linux는 아래 guardian 실증으로 보완했으며 Windows spawn→Job Object assignment 경쟁 조건은 아래 Windows 실제 adapter 실증으로 보완했다. 백그라운드 복귀와 OS logout/sleep 시험도 끝나지 않았다. [남은 게이트](../validation/P0-status.md) 없이 S03 완료를 주장하지 않는다.

## 2026-10-01 Linux 부분 결정과 증거

Linux 개발 adapter에는 작업별 guardian을 채택했다. argv-only 도구 실행과 anonymous socket의 readiness/TERM/KILL/EOF, subreaper 자손 회수를 사용한다. 직접 leader는 정리 이전에 reap하지 않아 PID 재사용에 따른 잘못된 kill을 피한다. 소유 leader가 PGID를 변경해도 직접 PID로 정리한다. 새 dependency/schema는 없고 GUI 바이너리를 내부 headless 모드로 재사용한다. engine 독립 개발 시험은 작은 helper binary를 빌드한다.

[검증 기록](../validation/guardian-validation.md): owner SIGKILL·중첩/setsid 자손·무관한 프로세스 생존·정상 leader 종료·그룹 변경·helper 누락, 실제 release headless entry와 자원 비용. 시작 journal 복구를 UI worker와 gate에 연결하고 손상/symlink 보존을 검증했다. 전체 프로세스 모델은 Proposed다. guardian 자체 SIGKILL/고장, OS logout/sleep, 다른 OS와 긴 작업은 미완료다. Linux /proc/subreaper가 없으면 안전한 weak fallback 없이 실패한다.

## 이력 재개와 추가 검증

재개 옵션은 argv를 역파싱하지 않고 버전 있는 typed JSON을 별도로 저장한다. 기존 snapshot과 일치해야 하며 옵션이 없는 legacy 작업은 재개하지 않는다. 새 Attempt와 원래 Job ID는 분리한다. 이력 UI와 옵션 실패 경로는 [작업 계약](../work-items/S03-job-resume.md)을 따른다.

[guardian 추가 실험](../validation/guardian-fault.md)은 TERM/INT/HUP/QUIT 정리 개선을 증명하지만 guardian SIGKILL에서는 분리된 자손 생존을 재현했다. 전체 S03 완료가 아니며 stream drain과 소유권 fallback은 후속 구현 대상이다.

## 다중 결과 공개 journal 개발 연결

Download adapter v5는 최대 3개 결과의 경로·크기·Unix native identity를 128KiB 이하 JSON 계획에 저장한다. 새 단일 영상 공개도 이 계획을 사용한다. 공개 전 전체 staging/충돌 검증, create-only hardlink, 전체 완료 transaction, 이후 matching staging 정리 순서다. Unix 재시작은 persisted identity로 일부 공개된 묶음의 누락 결과를 공개하고 한 번에 완료한다. 손상 계획이나 바뀐 파일은 Interrupted로 보존하며 legacy 단일 계획은 기존 복구를 유지한다. 비Unix identity와 actual 자막 metadata/UI, same-user raw path 경쟁은 아직 미완료다. [작업 계약](../work-items/S05-artifact-batch.md)을 따른다.

## Windows 원자적 소유 프로세스 생성

제품 adapter는 non-inherited kill-on-close Job을 CreateProcessW startup JOB_LIST에 지정하고 suspended child membership을 확인한 뒤 Resume한다. stdio handle allowlist로 Job·다른 작업 pipe 상속을 차단한다. 부모 종료 시 생성/별도 할당 사이 orphan 구간을 없애며 미지원 API는 fail-closed다. 주 담당은 코드의 Job 생성·startup attribute·membership·Resume·Drop을 검토했다. 실제 Windows GNU parent crash/cancel/timeout/정상 root 종료 및 literal Unicode argv와 실패 handle 수 검증은 [전용 증거](../validation/windows-process.md)에 기록한다. 자막 UI 포함 최신 Windows GNU 전체 fmt/clippy/test/release 재검증이 통과했다. 주 담당은 final evidence의 입력 파일47개 SHA를 현재파일에 독립 대조해 불일치0개와4명령exit0을 확인했다. Windows app13/Linux14 차이는 Unix 전용 metrics 시험1개이며 누락 UI source가 아니다. API 최소 Windows10/Server2016와 실증Windows11 하나를 구분하며 제품 최소 OS 정책은 Proposed다. Windows file identity·GUI/IME·MSVC/설치·sleep/logout과 전체 S03 게이트는 남는다.

## Linux 창 숨김·명시적 복귀 구현 결정 (2026-10-02)

Linux에서는 single-instance guard 뒤 private owner directory의 nonce 인증 restore-only Unix socket을 사용한다. 닫기 확인에서 명시적으로 선택한 경우 Iced Hidden으로 창을 숨긴다. 같은 앱 재실행만 Windowed 복귀를 요청하며 완료 이벤트는 창 표시·focus 요청을 하지 않는다. UI polling은 atomic flag와 capacity1 tick이며 긴 IPC I/O는 전용 worker에서 처리한다. endpoint 준비 실패/비지원 OS에서는 숨김 선택을 비활성화한다. 새 data directory는0700으로 만들며 기존 폴더 권한이나 외부 endpoint를 덮어쓰지 않는다.

트레이 의존성과 별도 daemon을 추가하지 않는다. 기존 창·renderer를 유지하므로 창 자원 해제 요구의 전부를 충족하지 않는다. Windows/macOS IPC·트레이·logout/sleep·최소 숨김 자원 예산은 남은 게이트다. 이 부분 구현 결정은 전체 Proposed 아키텍처의 확정을 뜻하지 않는다. [작업 계약](../work-items/S03-background-ui.md)과[가상 화면 검증](../validation/background-ui-validation.md)을 따른다.

## Windows 숨김·복귀 실증 갱신 (2026-10-02)

이후 Windows native GNU의 private desktop 합성 시험에서 일반 실행 인자로 숨김·동일 worker 유지·재실행 복귀·중단 후 종료를 3회 확인했다. 초기 synthetic 입력 실패는 harness의 mouse leave와 press 순서 문제였으며 제품 입력 우회 없이 harness 간격을 수정했다. [실제 증거와 snapshot 범위](../validation/windows-background-gui.md)를 따른다. 앞선 Windows 미지원·GUI 미검증 표현은 당시 기록이다. macOS·실제 IME·로그아웃·절전·트레이·숨김 자원 예산과 전체 architecture의 Proposed 상태는 유지한다.

## 목록 스캔 취소와 후보 종료 확인 (2026-10-03)

기존reconcile_library는유지하며optionalAtomicBool의협조적취소API를추가했다. 취소는directoryentry/최종publication/commit전경계에서확인하고transaction을rollback해기존snapshot을보존한다. 단일OS/SQLite호출강제중단은제공하지않는다. 실제시작전/관측중/commit전취소와후속freshscan을 [회귀시험](../work-items/S01-library-scan-cancel.md)으로확인했다. CPU파일목록panel은단일worker/channel1/128개page를사용하고panic에도bounded고정오류및stopack를전달한다. [CloseWait통합](../work-items/S01-cpu-library-integration.md)의actualGUI종료조건은추가검증중이다. 이정책추가는전체architecture Proposed상태를변경하지않는다.

## CPU 후보 event wake 검증 (2026-10-03)

인증된 복귀 IPC는 capacity1 pending flag와 선택적 wake callback을 함께 사용한다. callback은 UI repaint hint이며 작업 실행/상태 권한을 갖지 않는다. pending 이후 등록도 wake하고 반복 요청은 coalesce하며 callback panic은 pending을 보존한다. callback lock 밖 호출로 재진입 deadlock을 피한다. CPU 후보의 무조건250ms UI polling은 제거하고 실제 worker/IPC 이벤트로 깨운다. Linux real socket 회귀 및 callback-only b8bb snapshot의 active-hidden/complete-hidden 격리 GUI PASS를 주 담당이 원시 결과로 대조했다. Linux 단일 idle 표본 감소는 [측정](../validation/cpu-idle-regression-validation.md)을 따른다. Windows callback 변경의 native CLI 및 macOS는 후속 검증이며 전체 Proposed 상태는 유지한다.

## Linux 등록 앱의 사용자 서비스 보호 결정 (2026-10-03)

Linux등록앱(main·accessible·CPUentry)의기본guardian을기존사용자systemdmanager의작업별transientservice에서실행하도록채택했다. guardian은같은앱실행파일의headlessentry이며별도상시dlpaddaemon/adjacenthelper/관리자설치를요구하지않는다. launcher는앱의직접Child로pipe·wait계약을유지한다. 기존libc의libsystemd동적로드이므로새Cargo의존성/schema변경은없다. 기존사용자manager/library/runtime비용과앱/launcher/guardian비용은별도표본으로기록한다.

ExecStartEx no-env-expand로literalargv/환경을유지하고Typeexec·KillModecontrol-group·SendSIGKILL·Restartno를설정한다. 고유nonceunit/AddRef/동일bus/InvocationID/cgroup/PIDstartticks/pidfd를대조한다. guardian자체SIGKILL과앱+guardian동시소실에도manager가detached자손을정리한다. 능력실패에는process-group-only/direct로자동fallback하지않으며등록앱의명시direct선택도거절한다. 독립enginehelper의host-independent개발fixture는direct경로를유지하고strictnative시험은managercapability를명시선택한다.

[실제Rustengine13case](../validation/user-service-engine-integration.md)와[단독등록앱defaultGUI3case](../validation/user-service-app-default.md)의원시자료를root가독립대조했다. 13case의53PID/10cgroup과GUI3case의실제DB/동일앱guardianSHA/6workerguardianPID/3cgroup을확인했다. CPU기본GUI는Linuxea7595snapshot의합성자료이며실제mediaDecode/다른OS/8h를대표하지않는다. 현재사용자manager249.11실증은지원최소version확정이아니다.

시작후GetUnit/InvocationID실패에서서비스endpoint복제로cleanup이지연될수있어[시작실패보강](../work-items/S03-user-service-startup-cleanup.md)을추가중이다. 고정오류frame과소유socketshutdown으로미확인unit/PID를검색/kill하지않는다. 이후실제fault/release/Windows회귀와runtime비용·systemd없는Linux지원정책·logout/sleep는남는다. 전체architecture의Proposed와S03/P0미완료상태는유지한다.

## Linux guardian 초기화 정리 시간 경계

readiness실패뒤무한Childwait를제거하고5초자연종료→정확한unreapedChildkill→2초추가trywait를채택했다. spawn전8슬롯예약·singleton128KiB요청stackreaper·소유Child이관으로미회수자식을버리지않는다. 정상ready는예약반환,포화/thread생성실패는spawn전고정오류다. emptyworker는Condvar대기이며20ms관측은실제deferredChild가있을때만한다. 새dependency/schema없음. [실제19case](../validation/guardian-startup-wait-regression.md)와root단기unit/public/clippy를대조했다. 공개EP/noR의약5/15초는startup실패정리시간이며다운로드성공/취소latency가아니다. 전체workspace/Windows/release·추가thread비용·readiness중cancel/전체timeout은후속인수다. 정상ready뒤OwnershipDrop의별도blockingwait와전체S03/P0게이트는이부분결정으로완료하지않는다.

## 세션 인증의 개발 연결

실행 전용 SessionCookieLease를 DownloadSession으로 전달하고 typed options/argv snapshot/일반 DB에는 저장하지 않는다. 내부 artifact 검증 후 인증 실행 raw 출력과 자유 metadata의 외부 반환을 생략한다. 미확인 process cleanup은 typed marker로 구별하고 복사본을 보존하며 FD를 닫는다.

Linux 개발 adapter의 구현 범위이며 전체 architecture의 Proposed를 변경하지 않는다. guardian/service terminal acknowledgment, 시작 시 보류 복사본 회수, native GUI/비Unix ACL은 [미완료 게이트](../work-items/S02-session-cookie-download-integration.md)。

## Linux terminal 정리 확인 부분 구현

GC/GU guardian확인과LC/LU service확인을privateCLOEXECsocket의고정2byte프레임으로분리했다. direct는GC,service는GCLC와EOF를최대1초/5byte수신경계로검사한다. genuine도구exit126은GC+LC일때원래실패code로보존하고보호실패와혼동하지않는다. missing/unknown은typed미확인오류로sessioncopy보존에전달한다.

[root 독립22case](../validation/evidence/terminal-native-first-20261004/root-review.json)에서actualservice/rawprotocol·PID/cgroup정리를대조했다. 등록앱GUI/전체release/비용/다른OS/startupcookie회수와readiness전체deadline·정상Dropblockingwait는남는다. 전체architecture Proposed와P0미완료는유지한다.

## 2026-10-04 시작 단계 중단 계약

실행 bounds 검증 직후 하나의 absolute deadline을 생성하여 admission/readiness/실행 루프에 공유한다. 취소가 deadline보다 우선한다. Linux admission은 기존 8개 소유 슬롯과 1초 자체 상한을 유지하되 최대 20ms씩 취소를 관찰한다. readiness는 최대 2byte partial frame을 보존하며 R/EP/GU/LU 기존 문법을 따른다. spawn 뒤 preR 중단은 종료 미확인 typed 오류로 남겨 쿠키 reader 확인을 허용하지 않는다. 아직 Linux 전체 회귀 수정·공개 noR 실제 시험이 진행 중이므로 그 플랫폼 인수 완료를 주장하지 않는다.

Windows는 suspended child를 ResumeThread 직전에 확인한다. 이미 중단이면 소유 Job 종료, 유한 Child wait, Job ActiveProcesses=0을 모두 확인한 뒤에만 Cancelled/TimedOut을 반환한다. [실제 Windows 시험·Clippy](../validation/evidence/startup-checkpoint-windows-retry/root-review.json)가 이 경계를 확인했다. OS spawn syscall 자체의 hard deadline과 API 반환 총시간을 보장하지 않는다. 기존 단계별 유한 정리 예산은 실행 deadline에 추가된다. 새 dependency/schema/제품 thread는 없다.

## Windows 인증 실행의 독립 helper 연결 (2026-10-04)

Windows 인증 metadata/download는 CREATE_SUSPENDED+원 Tool Job을 보유하는 독립 headless helper로 연결했다. 등록 앱은 현재 앱 headless entry를 재사용하고 독립 engine 시험은 현재 build directory guard를 사용한다. helper는 TOOL Job 밖에서 exact5 inherited capability(control/events/receipt/stdout/stderr)만 전달받으며 PID 조회/Job breakaway fallback을 쓰지 않는다. 앱은 durable Bound를 commit하고 journal native lock을 넘기며 helper의 원 잠금 ACK 뒤 Begin을 보낸다. originalreceiptwriter는 부모가 duplicate 전달 직후 닫는다.

Windows 실제 app-like processkill 뒤 살아있던 helper의 원 Job 정리·native Terminal/exit·actual256receipt·원journal 회수를 검증했다. 제품 importer와 public recovery의 실제 helper receipt 회수 성공 및 foreign/source보존도 검증했다. 같은앱 GUI의 전체 crash/3OS/actualsite 인수와 새 downloader capture의 성공·실패·취소 및 전체workspace 검증은 남는다. 현재 연결 자체를 전체 P0/아키텍처 확정으로 표시하지 않는다.

새 dependency/schema 변경없음. 인증 reader마다 helper1개·4개 pipe·receipt capability 및 bounded256KiB config가 추가된다. 로그/metadata는 기존256KiB/8MiB 상한이며 제품 helper 실행 비용과 최신 GUI 메모리/8h 측정은 별도게이트다.

## Windows 관리 도구 reader의 별도 영구 기록 (개발 중)

쿠키 helper와 관리 도구 reader는 소유 파일과 도메인이 다르므로 기록·receipt와 제어 magic을 분리한다. 관리 제어 프레임은 DLPMG001, 8개 단일 사용 슬롯이며 cookie DLPWG001의 2개 슬롯 계약은 유지한다. Windows 실행 시작은 exclusive 관리 잠금 아래 원 journal·receipt 생성과 v3 marker의 immutable anchors/base digest를 sync한다. 중간 실패 기록과 legacy v2는 보존한다. 원 객체의 pristine 정리는 journal 파일 실제 부재 확인 후 marker-last 순서다. 정상 reader 종료 증거와 startup 자동 회수 연결은 미완료이며 이 부분 구현은 전체 Proposed 상태나 P0 완료를 바꾸지 않는다. 기존 의존성만 사용하며 실행 세션당 journal 1개·고정 receipt 8개와 bounded16KiB 기록이 추가된다. [작업 계약](../work-items/S04-windows-managed-reader-lifecycle-integration.md).

## 관리·인증의 단일 Windows Job 결합 (부분 실제 인수)

관리+인증 metadata/download는 별도 headless paired entry의 정확히6개 원 capability로 연결한다. DLPPG001과 두 원 Bound의 domain-separated digest를 사용한다. 각 journal의 독립 잠금 확인 후에만 원 Tool Job을 Resume하며 Job 종료 확인→두 receipt sync→양 잠금 drop→Terminal 순서를 따른다. cookie-only/managed-only는 기존5개 capability 계약을 유지한다. 공개 합성 다운로드 성공과 원본 보존·DB privacy·정리·재사용, 전체 Windows 회귀를 실제 확인했다. 실패·취소·crash/startup 회수와 제품 GUI 인수는 후속이며 전체 Proposed/P0 상태는 바꾸지 않는다. 기존 의존성으로 helper1개를 유지하고 추가 receipt HANDLE1개와 bounded256KiB 결합 config를 사용한다. [인수 범위](../work-items/S04-paired-download-native.md).

## Windows 원 관리 실행 시작 복구 (부분 실제 인수)

Windows는 기존 MarkerBinding 단일 권위로 canonical v2 digest를 재구성한 v3 seed를 검증한다. no-create 원 install lock의 exclusive 권한과 late original marker DELETE 권한, 원 journal 잠금/실제 전체 receipts를 모두 얻은 경우에만 journal/8 receipts consuming cleanup 후 marker-last/실제 부재를 확인한다. pristine/legacy/Intent/Busy/손상은 보존한다. 공개 paired 합성 부모 종료73 뒤 두 도메인 독립 회수·DB Interrupted·원 Job 자연 종료와 원본/foreign 보존을 실제 확인했고 실패 보존 8회귀 및 Windows 전체 네 검증도 통과했다. 앱 worker 호출 연결은 후속 최신 검증 중이다. 기존 bounded 파일과 단일 worker를 재사용하며 새 dependency/schema/상시 process는 없다. recovery 중 유한 원 HANDLE을 열고 4096/16384/256바이트 상한으로 읽는다. 전체 Proposed/P0 및 GUI/실제 사이트/패키징 완료로 확대하지 않는다. [작업 계약](../work-items/S04-managed-startup-recovery-integration.md).


## Windows 관리 Deno 환경 전파의 초기 통합

원 exe의 node shim hardlink를 피하는 공식 opt-out은 offline native에서 검증했다. typed private-field policy와 bounded Unicode encoder 및 원 CreateProcess 환경 전달을 초기 연결했고 None 호환을 유지한다. 소유 cache·관리 Deno 분류·helper wire 전달은 아직 통합 중이라 production에서 임의 활성화하지 않는다. 상속 환경 검사에서 실제 native 회귀가 실패해 원인 계측 중이며 전체 architecture/P0 또는 이 기능의 인수 완료가 아니다. [작업과 원 실패](../work-items/S04-runtime-environment-propagation.md). 의존성 추가는 없고 Some 실행 시 최대32767 UTF16 단위 block을 준비한다; 성능 실측 완료를 주장하지 않는다.


## Windows Deno 환경의 실제 회귀와 원 cache 기반

앞선 초기 환경 인코딩 실패는 inherited leading-equal pseudoentry 보존으로 수정했고 typed direct spawn 및 strict helper wire의 Windows 전체 검증을 통과했다. 이는 cache 원 권한이나 실제 guardian Some 실행 완료를 뜻하지 않는다. 관리 Deno 분류는 명시 비활성 패키지의 전체 bytes/metadata/native pins를 확인하며 관리 yt-dlp 선택 여부나 active pointer로 소유권을 추정하지 않는다. 새 cache 기반은 기존 manager root ACL을 수정하지 않고 새 runtime/cache/owner/lock에 current-SID 보호 ACL을 부여하며 원 별도 lock과 bounded owner record를 유지한다. unknown/partial은 보존하고 자동 삭제하지 않는다. 원 cache 권한의 helper handoff 전에는 생산 Some 거절을 유지한다. 의존성·DB schema 변경은 없으며 추가 비용은 retained HANDLE, 4096-byte 이하 owner record와 최대4096 entry의 사후 metadata 조사다. 논리32MiB 예산은 디스크 hard quota나 메모리 실측 보장이 아니다. 실제 payload ACL/guardian 비용/공급 채택/P0 전체 및 architecture Proposed는 별도 인수다. [분류](../work-items/S04-managed-deno-classification.md), [cache](../work-items/S04-owned-deno-cache.md).

## Windows cache 원 capability의 gate 인계 부분 채택

원 cache.lock HANDLE 하나와 typed original cache attachment를 전달하고 app explicitunlock→helper 자체 잠금/Ack→Begin으로 인계하는 구현을 채택한다. DuplicateHandle은 app 잠금 이전 증거가 아니다. root/runtime/cache/owner/lock 원 IDs와 bounded rawownerSHA/보호SIDACL을 재검증하고 no-create 원 namespace pins를 helper terminal까지 보유한다. optionalcache append 뒤 stdio는 원 base indices를 유지한다. 실제 cookie helper Some에서 원 app HANDLE close 뒤 Busy/terminal 및 재획득을 확인했다. [root 전체 실제 증거](../validation/evidence/windows-deno-cache-handoff-workspace-first/root-review.json). 실제 appprocesskill·managed/pairedSome·생산 factory/다운로드·cachepayloadACL은 별도게이트이며 overallProposed/P0 상태는 바꾸지 않는다. 기존 Cargo/dependencies/DBschema로 원 capability1개와 bounded original attachment/원pins만 추가하며 상시daemon은 없다. 사후32MiB는hardquota가 아니며 peakRSS/전체처리시간 실측 완료를 주장하지 않는다.

## Windows 관리 Deno 제품 연결과 retained 원 검증

원 cache authority와 VerifiedManagedDeno는 Arc로 실행 수명에 보유하며 shared DB/options에 저장하지 않는다. 실제 Windows public 합성5case 및 공식 Deno independent managed helper3case가 원 receipt·cache·exe identity를 보존했다. 실행 scope 원root와 environment cachebinding의 path/nativeID가 일치해야 하며 원 notice/component/metadata bytes는 factory 이후와 각 capture journal 변경 전에도 bounded full-set 검증으로 다시 확인한다. 기존 read-only verifier를 원 root HANDLE 기반으로 재사용하고 DB 생성/다시열기나 경로 기반 권한 재생성은 하지 않는다. 공개 DownloadSession preflight는 첫 yt-dlp version probe 전에 selected Deno와 원 proof를 검증한다. 추가 dependency/schema는 없고 각 capture에 원component64KiB streaming hash IO가 추가된다. bounded deadline/cancel은 유지하며 공식3stage fixture 전체8282ms는 helper·재hash 비용 포함, 독립 RSS/처리비용 예산 확정이 아니다. 공급 채택·actual site/GUI/부모kill 및 전체 architecture Proposed/P0 게이트는 유지한다.

## 2026-10-05 Windows scan CloseWait 관측 범위

협조적 scan cancellation과원DB snapshot보존 계약을 유지한다. 기본OFF fixture에서 첫 ordinaryfile observation INSERT 이후 publication전의원event를사용해 stop의 실제진행지점을관측한다. 새행동/도메인jobstate가아니며개별Win32/SQLite syscall 강제취소보장으로확대하지않는다. 실제Windows privateGUI release/20초timeout은 ACK전원app유지→rollback/worker반환ACK→자연exit0/Job0를확인했다. root가원DB1행/nativeidentity·신규파일미공개·sentinel과PNG를독립대조했다. [작업기록](../work-items/S03-windows-scan-closewait-native.md). fixture/기본제품최종전체검증은해당최신기록을따른다.

## 2026-10-05 Windows 목록 batch 후보 통합

기존10k 재확인 CPU약1.17초와주기GUI4.6~5.3%를관측했다. 원native directory128bitID batch 실험 및모든1만file ID/size/mtime/bytes 대조를근거로Windows 후보scanner를통합한다. 초기지원preflight→원rootpin유지restart→streamed관측이며 미지원reparse/filesystem은보수scanner를사용한다. batch/child/commit경계에서원native directoryID·finalpath를검증하고원directory rename을share정책으로거절한다. 각depth64KiB×최대128frame=8MiB buffer상한과원rootguard HANDLE1개가추가되며 전체목록을Vec에모으지않는다. dependency/schema/Cargo변경0.

Kernel batch는개별fileHANDLE 검증과동등한현재namespace권한이라고주장하지않는다. 파일변경은관측뒤에도일어날수있고 기존watch/30초 재확인으로조정한다. 공개경로rename후file교체경합을실제red로재현했고 unique rename후보의path/size/mtime/128ID를레코드이전직전에기존perfile방식으로확인하여rollback하도록보강했다. 원rename유일성·hardlink보존 SQL은유지한다. 실제Windows목록회귀22개와전체ID대조는PASS지만첫fullClippy2경고는보존·교정후재검증중이며 최신GUICPU예산은아직인수하지않았다. 전체아키텍처와최종S00선정의Proposed상태를변경하지않는다. [작업계약](../work-items/S01-windows-native-library-scanner.md).

## 2026-10-05 정확 동일 목록 publication 생략 후보

Windows native 관측의 root전체행수·유일path·size/mtime/opaqueID/present/SQLite storage class가 기존 DB와 모두 같을 때만 rename 후보 검증 및 UPDATE/UPSERT/DELETE를 생략한다. 동등집합에는 missing-old/unused-new 후보가 존재하지 않으므로 기존 record-transfer 조건을 완화하지 않는다. observations clear·취소·root HANDLE 검증·commit은 동일하다. 실제 실패 trigger·같은개수 변경/NULL/present0·hardlink 회귀와 원GUI 기준으로 인수하며 단순 count 비교는 금지한다. 새 default-off 계측feature는 dependency0/기본 runtime0, 계측실행은 scan당 약20회CPU조회·고정9단계값만 기록하고 제품 예산판정을 대신하지 않는다. 스택/S00 전체채택은 여전히 별도다.

### 동일 배경 목록의 UI 전달

worker의 마지막 성공 전달 root/rows-or-error/health와 정확히 같은 자율 결과는 큐 게시·UI wake를 생략한다. 사용자 명령 serial 또는 root ACK는 항상 전달한다. 마지막 키는 send 성공 후에만 갱신하며 취소·채널 종료·큐 경합 실패로 덮지 않는다. 최대128행은 Arc<Vec<LibraryEntry>>로 공유해 UI/마지막 키의 전체 경로 복제를 피한다. 소비되지 않던 library private free 조회를 제거하고 화면 볼륨 값은 MetricsPanel의 독립 root revision/최신 표본 관측을 유지한다. DB/schema/재확인 주기는 바꾸지 않는다. 실제 Windows 회귀와 GUI 비용을 따로 검증하며 코드 변경만으로 성능 목표 달성을 선언하지 않는다.

### 일반 dirty hint의 fixed deadline

실제Windows partial append세회가동일1만목록 fullscan을각각유발했다. lexicalpartial필터는hardlink/reparse관계의일반파일변경을안전하게분리한다는권한이없어채택하지않는다. WatchSession의모든일반native/rootmtime힌트를pendingbool+고정1초next_due로합친다. health/backenderror/rootidentity손실/attach/30초보완은gate밖에서확인하고urgentdirty로flush한다. 기존rootmtime→30초deadline갱신·atomic도착힌트·manual명령·취소정책은유지한다. 추가thread/queue/deps/schema0이며단순상수크기timer가추가된다. 실제nativeburst/alias/최종Create/실패복구와전체GUI를검증하고DBcommit≤1초나idleGUI원인을추정하지않는다. [작업계약](../work-items/S01-watch-hint-coalescing.md).


## 2026-10-05 hard-link 메타데이터 보정

원 burst 시험과 watcher 없는 1파일 최소 시험에서 alias12→13B 뒤 DB12가 남는 오류를 재현했다. metadata 사전 조회 경우만13B였고 원128ID는 같았다. [Microsoft hard-link 설명](https://learn.microsoft.com/en-us/windows/win32/fileio/hard-links-and-junctions)의 link별 directory metadata 갱신 특성과 일치한다. 따라서 기존 batch EOF/time publication을 수정하여 모든 regular entry에서 최신 metadata를 조회한다. 이름·volume/full128ID는 enumeration에서 유지하고 반환원ID와 반드시 대조한다.

Windows11 24H2 Basic-by-name API는 System32 API-set 동적 로딩으로 선택 사용한다. static import·최소OS 변경·새 Cargo dependency는 없다. DLL/symbol 없음 또는 preflight의 정확한 unsupported class는 원 no-reparse HANDLE StandardInfo/BasicInfo/full128ID로 내려가고, permission/notfound/reparse/ID mismatch/transaction 중 오류는 rollback한다. bounded UTF16 path와 SDK104B ABI를 검증하고 loader reference는 scan 수명 안에서 해제한다. query는 snapshot이며 원frame/root guard·cancel·unique rename 재검증·최종commit 경계를 유지한다.

첫 정확성 보정은 두 pass에서 파일당2회 query를 수행했고 실제1만파일 scan CPU가0.61~0.64초였다. 후속으로 preflight namespace 순회와 metadata publication seam을 분리했다. preflight는 optional class를 첫regular파일에서 한 번 확인하고 결과metadata를 반환하지 않으며, SQL pass의 모든regular파일은 fresh query를 유지한다. 예상 query는2N→N+1이며 원cancel/root/depth/reparse/ID 검사·late오류rollback은 동일하다. 실제 API alias 및 첫정확성1만burst 시험은 통과했지만 중복축소 후 비용·최신GUI·구형OS 실행·symbolic-link 경쟁 인수는 별도다. 기존 multi-rename 시험의 'batch가 모든파일크기를 이미 관측했다' 전제는 새 per-file 관측 시점과 다르므로 이미 관측한 파일을 확실히 수정하는 회귀로 보강한다. 오류나 이전DB전체보존 assertion을 완화하지 않는다. 전체 스택/S00 Proposed 상태는 별도이며 [작업 계약](../work-items/S01-windows-hardlink-metadata-freshness.md)을 따른다.

## 2026-10-05 metadata query와 polling pacing의 미채택 실험

원directory HANDLE의 NtQueryInformationByName Basic class77을 isolated fixture에서 실증했지만 1만query 양방향6측정은 기존Win32 Basic-by-name과 CPU0.265625~0.28125초로 차이를 입증하지 못했다. 미지원/STATUS_PENDING 및 user-mode 지원 범위 복잡성을 더할 근거가 없어 제품에는 도입하지 않는다. full128ID와 freshmetadata 조회를 유지한다.

기존FnMut checkpoint의default-off 1ms/250ms 계획·10ms polling sleep 실험은 전체138.417초/CPU0.75초·alias13B/ID·원DB/source 보존을 확인했다. 한10초구간1.24995% 및 빠른scan보다 높은 총CPU 때문에 제품 미채택이다. 느린완료를30초동일workload 예산달성으로 바꾸지 않는다. event wake 대안과 불필요한Next fullscan 제거는 각각 별도실험/수직작업으로 검증한다. [실험계약](../work-items/S01-periodic-scan-pacing-experiment.md). 전체 architecture와S00 선택은 Proposed를 유지한다.

## 목록 Next의 bounded DB 조회 분리

Next가 전체 reconciliation을 반복하던 경로를 마지막commit snapshot의128행 keyset 조회로 분리한다. root/data canonical분리와 취소·단일worker·serial ACK는 유지한다. 외부수정은watch 또는 명시Refresh로 반영하고 Next 자체가 최신fullscan을 약속하지 않는다. 원140행시험에서 변경metadata와추가파일이 즉시DB에 반영되는 기존경로의red를 실제Windows에서 확인했고 분리후원회귀는green이다. 260행 전체키셋·Stop/corruptDB/겹치는root의실패인수와 최신전체/GUI는 별도다. 의존성/schema/lock 변경없음. [작업계약](../work-items/S01-library-page-read-only.md).

### Readonly 페이지 실제 인수 갱신

Store::read_library_page_readonly는READ_ONLY/NO_MUTEX/NOFOLLOW·기존5초SQLitebusy설정으로기존파일만열고from_connection/migration을호출하지않는다. 앱은root/data분리·일반directory/Windowsreparse거절을먼저확인한다. actualWindowsmissingred→green·두앱19회귀/engine2회귀·최신workspace4검증과8f21binary의격리Next/Refresh/invalidroot GUI가통과했다. root가원PNG/UIA/DBcursor128·원fixture의도된변경외payload보존을검토했다. scope완료와GUI idleCPU/물리IME·reader/설치/최종architecture게이트는분리한다.

### 2026-10-06: 감시 root 관측 재사용

WatchSession의 한 active poll에서 identity와 mtime를 동일 Metadata 관측으로 얻는다. attach 전후 identity 검사는 유지하고 성공 후의 mtime를 재사용한다. 100ms 호출, ordinary1초, 재연결2초, 보완30초 정책은 유지한다. snapshot은 namespace lifetime 권한이 아니며 Windows created(None)의 기존 identity 한계를 강화한 것으로 해석하지 않는다. 실제 Windows red(2 vs1)와 watch10회귀 green은 통과했으나 성능 예산은 별도 측정한다.

### 2026-10-06: 라이브러리 관측 비용 축소

동일 quietworker의 60초 관측에서 두 full scan이 각 약0.344 CPU초를 사용했다. TEMP identity 인덱스는 publication이 필요할 때만 생성하고 setup에서 TEMP 인덱스만 제거한다. 초기 비교/path PK·main identity 인덱스·취소 rollback은 유지한다. 재사용 Store의 변경 없음→rename→취소→복구 회귀를 포함해 Windows library17회귀가 통과했고, 한 쌍의 후속 비용은 평균1.327%→1.248%였다. P0예산은 미달이다.

추가로 Api당 최대32767 UTF16 units의 경로 scratch를 재사용한다. 명목65534B와 allocator overhead는 walker 수명으로 한정한다. prefix/UNC/verbatim/NUL/길이 경계와 원 full128ID/no-reparse fallback을 유지하고 metadata를 재사용하지 않는다. Windows library27회귀와 fixture Clippy는 통과했으며 실제 CPU·메모리 인수는 별도다. 아키텍처 전체 채택으로 확대하지 않는다.

### 2026-10-06 주기 스캔 예산 실험 후보

최신 full fresh worker 평균1.2222% 미달을 근거로 periodic에만 core 순수 wall 예산2ms/500ms·최대180초를 연결한다. 1ms/500ms의 이상적164~172초는 기한 여유가 작아 후보를 변경했다. 실제 GUI strict10초/평균CPU·완료지연·Next/Stop/Recovery 인수 전 전역스택·성능 채택을 확정하지 않는다. engine checkpoint는 원 cancellation rollback을 재사용하고 typed cause로 일반 변경/복구가 느린 확인에 종속되지 않게 한다. 외부 도구·Cargo/schema/lock 변경은 없다.

### 2026-10-06 실제 thread CPU 예산 후보

1500µs/500ms 후보의 실제 GUI3회 평균0.52034/0.52011/0.38997%와 max10초1.09140/1.09192/0.77913%에서 strict2FAIL을 확인했다. wall폭만 줄여 완료로 처리하지 않고 동일 worker의 GetThreadTimes 누적 CPU와0.4% 부채 대기를 추가한다. reader는 원 thread ID/creation·합산 overflow·역행을 검사하고 !Send/!Sync이며 기존 windows-sys 기능만 사용한다. core 계산과 native reader를 분리하고100ms sampling/대기 및180초 전체 기한을 유지한다. 실제 전체/native GUI/Next/Stop/완료 인수 전 좁은 후보로 유지하며 전역 아키텍처·스택은 Proposed다. 누적 idle 여유/15.625ms급 counter 해상도/개별 blocking 호출로10초 quota는 보장하지 않는다. Cargo/lock/schema/추가thread 변경0.

### CPU counter 후보의 작업 폭 비교

동일private실험의actualcoreCPUbudget/GetThreadTimes/WatchSession·connectedidlechannel에서1500µs/500ms는180.008초/0.671875CPU초에deadlinerollback했고2ms/500ms는146.459초/0.578125CPU초에완료했다. 원10000payload/sample2nativeID/128행보존을확인한실험근거로작업폭만2ms로조정한다. 이는GUI/App전체완료나최종스택채택이아니며0.4%·180초·1%인수기준을완화하지않는다. CPUfirst합성반례의private초안은아직실행/채택하지않았다.

### 2026-10-06 CPU raster 재사용 실험

실측2ms/0.4% GUI의첫10초에서main0.7786%와library0.3114%가원Job1.0906%를구성했다. main의세부원인은미확정이다. 기존software renderer caching은추가canvas비용을피해비활성으로유지하며root가정확한원primitive/texturedelta/scale/size와surface age1을조건으로fill/raster/mask만재사용하는최소실험을통합했다. retained2MiB·추가pixel buffer/thread/dependency0, egui/접근성/platform/전체present를유지한다. 현재whole/native인수중이며실험을채택완료나성능PASS로표현하지않는다.

### 2026-10-06 quiet watch 대기 후보

동일frame후보GUI의한10초창이1.2526%로미달했고그창library78.125ms CPU가관측됐다. idle/scan원인은미확정이다. root는quiet recv_timeout만100→500ms로낮추고기존cap1command에StopWake를try_send하는최소실험을통합한다. Stop Release bit는wake전에설정하고receive뒤check해DB/serial처리전에끝낸다. full/disconnect는blocking하지않는다. quietwatch전달은최대500mspoll추가지연으로바뀌며scan중100ms/180초와30/1/2초내부gate는유지한다. Native전체·CPU·실제Next/Stop/완료인수전으로Proposed를채택완료로표현하지않는다.

### 2026-10-06 CPU eligibility 조합 후보

quiet500/StopWake/frame후보3ca의GUI3회는원strictCPU1%/80MiB를통과했지만동일source fixture의실제completion은180초commit미관측FAIL이었다. 원10kDB·CAP·sentinel은보존했다. CPU2.4ms가600mswall을요구하는가상반례에서기존wall-first는CPU부채대기중500ms작업창을소모한다. root는원2ms/500ms·4/1000·100ms·180초를유지하고CPUeligible 이후wall창을여는공유순수조합을통합했다. 전체deadline을먼저검사하고counter실패는원rollback한다. 실제원인확정과nativePASS는아직없으며no-bank별도초안은미채택이다.


### 2026-10-06 스캔 중 감시 조회 cadence 후보 구현

현재 구현은 scan watch.poll을 SCAN_WATCH_POLL=500ms로 제한한다. quiet 대기는 기존 500ms이며 Stop/명령 확인·CPU 샘플·대기 조각 100ms, wall 2ms/500ms, 누적 CPU 4/1000, 전체 180초와 감시 내부 30/1/2초 정책은 유지한다. native 이벤트와 root identity 확인을 생략하지 않는다. 감시 인식은 500ms+checkpoint/OS 지연이 가능하며 DB 반영의 hard deadline을 뜻하지 않는다. 동일 단독 바이너리 C1 137.071초/CPU 0.546875초와 A2 151.399초/CPU 0.593750초가 모두 완료했지만 이전 A1/B1과 실제 GUI 완료 실패를 보존한다. 실제 제품 완료·Next·Stop·root 복구·GUI CPU 검증은 진행 중이고 스택의 Proposed 상태를 채택 완료로 바꾸지 않는다. 추가 의존성·thread·schema·Cargo 변경은 없다.

후속 실제 검증: watch500 snapshot의 completion145.160초·Next2.031초·Stop ACK5초/commit금지·watch-only 자동 변경·startup missing-root2.010초와 GUI60초3회/18구간 CPU<1%·80MiB를 확인했다. 원 DB/payload 검증 범위는 [작업 기록](../work-items/S01-scan-watch-poll-budget.md)을 따른다. paced scan 중 실제 higher-watch preemption과 완전한 thread 원인 귀속은 미검증이며 최종 UI 스택의 Proposed 상태는 유지한다. 후속 설치 기반 소스는 해당 snapshot 인수와 별개다.


## 2026-10-07 Windows 복귀 listener 유휴 대기

개발 후보의 Windows server connect는 20ms stop Atomic polling 대신 원 overlapped 완료와 별도 manual-reset stop event를 함께 기다린다. stop을 index0에 두어 둘이 동시에 signaled일 때 취소를 우선한다. 두 OwnedHandle은 pending wait/drain보다 오래 살며, 종료는 stop Atomic→SetEvent→원 pipe CancelIoEx→worker join 순서다. 클라이언트/짧은 handshake의 기존250ms deadline 정책은 유지한다. 새 thread·crate·프로토콜·schema를 추가하지 않고 server당 event handle1만 추가한다.

[WaitForMultipleObjects의 인덱스와 handle 수명](https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-waitformultipleobjects), [CancelIoEx의 완료 대기 요구](https://learn.microsoft.com/en-us/windows/win32/api/ioapiset/nf-ioapiset-cancelioex)에 따라 기존 Pending Drop의 원 GetOverlappedResult drain을 유지한다. 실제 연결 없는120ms 구간에서 기존 native wait4회가 실패했고 event 방식1회로 회귀가 통과했다. 전체 CPU 기준 개선은 별도 실측한다. [범위와 검증](../work-items/S00-windows-restore-idle-event-wait.md).

### 2026-10-12 새 bundle 루트의 생성 충돌 실험

기존 CLI smoke는 정확한 AlreadyExists 거절에도 생성 전 보수적 mutation=true가 남아 실패했다. root는 원 FILE_CREATE의 정확한 NT 충돌 상태를 내부 typed 증거로 전달하고 이 경우에만 bundle 변경 가능성을 false로 보고하도록 보강했다. transaction의 target hint는 factory 호출 전에 설정되므로 상위 OR도 같은 정확한 충돌 증거에서만 제외한다. 다른 오류·소유권·취소 정책은 유지한다. 실제 Windows adapter133회귀가 통과했지만 최신 전체·CLI의 payload/native ID 보존 대조는 아직 남는다. 이 기록은 upgrade/rollback·서명·전체 릴리스 인수가 아니다. [검증 기록](../work-items/S07-existing-root-refusal-20261012.md).
