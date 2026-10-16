# 2026-10-04 현재 P0 남은 인수 감사

판정: **P0 전체 미완료**. ROADMAP/S10 범위를 바꾸지 않았다. 실제 통과한 scoped 기능과 첫 릴리스 게이트를 분리한다. 읽은 기준은 REQUIREMENTS R01~R13, ROADMAP S00~S10, P0-status, ADR-0001, 현재 work-item 및 실제 result/root-review/source manifest다. 새 감사 문서 두 개만 작성했고 코드·Cargo·기존 문서·프로세스·사용자 데이터는 변경하지 않았다.

## 먼저 처리할 실제 차단 사항

| 순서 | 아직 남은 구체 인수 | 현재 근거와 공백 | 다음 가능한 작업 |
| --- | --- | --- | --- |
| 1 | 같은 boot 보류 쿠키의 안전한 다음-start cleanup / Windows crash 잔존 | Linux v1 record는 sameboot를 항상 deferred,128 capacity에 도달하면 CacheFull. 메모리 reader guard와 previousboot-only cleanup은 구현·시험 완료지만 지속 reader intent/terminal receipt는 없음. Windows recovery는Unavailable. [선택된 pre-B 계약](../work-items/S02-cookie-sameboot-reader-gate-review.md)은 **검토 완료**이며 ReaderObserver/I→durable bind→B→R와 launcher cleanup-only receipt를 아직 구현했다는 근거가 없음 | root가 공유 process/cookie protocol을 조정하고 별도 파일 담당을 나눠 crash-before/afterB, detached-reader, manager-unavailable, late-empty receipt 실제 인수. Windows는 신뢰할 boot/Job receipt 계약 없이 자동 삭제하지 않음 |
| 2 | 최신 인증 Windows GUI 및 최신 startup 소스의 통합 회귀 | Windows 실제 ACL/tool read-write/cleanup, auth repository4mode, writer failures3, suspended pre-resume cancel은 scoped PASS. [private 인증 GUI](../work-items/S02-windows-cookie-private-gui.md)는 아직in-progress이고 build 성공은 UIA 실제 입력/시작/privacy/CloseWait 인수가 아님. Linux public noR4는 current process SHA에 scoped PASS지만 마지막 Linux 전체7명령은 그 전 snapshot | 현재 불변 Windows release와 합성 example의 HWND/UIA capability부터 success/failure/cancel 실행. root 새 fullfmt/default·CPUclippy/test/release와 최신 manager terminal/startup 회귀 후 immutable Linux GUI success/cancel/failure/close 재검증 |
| 3 | S00 최종 UI/renderer 채택을 위한 실제 입력·reader·theme·성능 | CPU 후보는 기능 통합 방향이고 ADR-0001 최종선택은Proposed. Linux 한글/무음Orca와150/200% settings 접근은 scoped PASS. Windows ASCII/WM_CHAR·UIA API 시험은 실제IME/reader가 아님. System OS 변경/첫 프레임과 모든필수화면 keyboard-only/진행률 알림은 미검증 | Linux 현재snapshot에서 main/settings/library/tools/history/report의 keyboard·reader·System 실제변경·첫paint 재현. WindowsTSF/reader는 사용자desktop 분리 가능한 실제 환경 필요, macOS 호스트 필요. 이 증거와 동일기능 비용으로 ADR결정 |
| 4 | R04/S03 긴 작업·복구의 실제 terminal 및 실패 경계 | 기존boot 시험은 약5h59m 중단. 새 archive는 아래 시점watching/두terminal null이며 아직8h완료증거 없음. 실제 EFBIG와 injectedENOSPC는 PASS지만 실제disk-full/SQLite·publication ENOSPC/sleep/resume·사이트서명만료는 별도 | 기존owned8h는 restart/kill 없이 root가 최종28800초·DBreopen·artifact·reap/archiveterminal 감사. 안전한 owned storage fault fixture와 suspend/resume capable 분리환경 설계. 현재guardian/backend를 같은 oldsoakhash로 검증했다고 쓰지 않음 |
| 5 | S04/S05 실제 FFmpeg/EJS 도구 공급과 실행 조합 | 고정yt-dlp 실제 install/cancel/use/remove LinuxGUI는PASS. BtbN LGPL FFmpeg/Deno pin은 Linux합성 decode/cut/JS 실행 후보까지만PASS. multi-component managed activation/rollback/source·notice/trust/OS조합 및 bundled EJS가 실제 pin에 연결됐다는 인수 없음 | 현재허용범위에서 archive installer/notice 수집과 synthetic toolchain 전체·빠른/정확cut·자막/품질 version matrix 검증. 외부FFmpeg 선택이 구현됐다는 사실로 ROADMAP S04 공급 게이트를 삭제하지 않음 |
| 6 | S10 P0의 Windows/macOS 로컬 report preview/copy/save와 안전한 clipboard 경로 | Linux paged1.9MB report fullcopy/export/collision/privacy/job보존은PASS. Windows 현재private clipboardstation 생성은ERROR_ALREADY_EXISTS183로실패, roundtrip/UIcopy는미검증. 모든OS source-size 메모리상한은 서로다른binary근거임 | Windows private reportpreview/save는 clipboard와독립진행. clipboard는 새reviewed 격리경로가 생기기전 해당시험만보류하고 WinSta0 fallback금지. macOS clipboard/reader는OS환경필요 |
| 7 | S01/S06/S07 저장·목록·3OS 설치·CI·배포 인수 | Linux10k/128page/watch/invalidroot/CloseWait는scopedPASS, Windows loadedrow GUI/redirectedDocuments/권한실패는별도. 정상Git/remoteCI없음, macOS/MSVC 미구비, 패키지·서명/upgrade·제거 인수없음 | 합성Windows loadedlibrary/rootfailure·현재SQLite/schema회귀, unsigned OS패키지 재현준비는진행가능. 실제 redirected profile/macOS/MSVC/remote·서명은 아래 환경·승인경계를 따름 |

같은boot 쿠키 보존은 안전한 실패 정책이지만 INTEGRATIONS의 next-start cleanup 요구 전체를 만족하지 않는다. Windows 자동recovery unavailable를 성공으로 바꾸거나 파일명/mtime/PID/unitGC로삭제하는 우회는 금지한다. 인증rawoutput 자유텍스트 생략·원본보존·동일세션copy 정책은 계속 유지한다.

## 현재 증거와 snapshot 구분

| 증거 | 실제 완료 범위 | snapshot / 현재에 확대할 수 없는 이유 |
| --- | --- | --- |
| [Linux 전체7명령](evidence/owned-child-reader-workspace-retry/workspace-result.json) | fmt/default·CPUclippy/test/release 모두exit0, OS공통parser6·ownedChild·readerguard 시험 | CPU `d0451487a8127f6891ccf377e8aa1826fe9261b4c554e6278c791b23a3d9c3a5` /12,515,320B. source-end269manifest와 감사시 current는 accessible.rs/cookies.rs/process.rs/process/windows.rs/session_cookies.rs가 달라 최신전체PASS로 확대불가 |
| [Linux library retry](evidence/startup-checkpoint-linux-lib-retry/result.json) |198PASS/0FAIL/2ignored + libraryclippyexit0 | process.rs `04fd175181c244db0f9359d12e7914fec38ce30a4142093708c310710359cc38`가 아래감사시값과일치. lib시험이지GUI·전체workspace·manager실증아님 |
| [public startup4](evidence/startup-interrupt-native-first/root-review.json) | noR150ms/관측후cancel/partialE/metadata, EOF0.111–0.143sec, 반환5.17–5.22sec,FD4→4,정확childreap | driver SHA `fb9cb0dd1c31bb833f833bd529e52cf75e974b003f5a920c33a2faedf7343706`; probe `b95897f46a544b4ccf15e01ddd49cce1ac1ef77b5023335e5f2fda29b7f4022f`. fake adjacent helper/directdevelopment 경로이며 실제cookie/servicecgroup 아님 |
| [Windows startup/writer retry](evidence/startup-checkpoint-windows-retry/root-review.json) | CPUworkspace test/clippyexit0, nativewriter3와pre-resumeinterrupt | 273inputs 당시before/aftermatch. 감사시 process.rs 하나가manifest와다름(주로Linux변경이어도현재전체입력일치라고쓰지않음). 명시authintegration은ignored였으므로auth4mode 별도재시험 |
| [Windows auth4](evidence/windows-authenticated-download-first/root-review.json) | success/metadatafail/downloadfail/cancel,원본·privatecopy·DBprivacy | 당시273inputs matchreview. 실제사이트/GUI/crashrecovery아님,후속processcheckpoint와구분 |
| [Windows CPU release](evidence/windows-cookie-cpu-release-current/root-review.json) | actualGNU buildexit0 | SHA `c605b9e8f0f242f706ff7948daac74e9c82d9950e09b175d050f1bfdb6fda946` /7,285,248B. 감사시windowscookie/processwindows/process 소스3개는buildmanifest와다름. 새releaseREADY전후를구분 |
| [Linux settings150/200 matrix](cpu-scaled-native-validation.md) |3테마×2배율 실제wheel/IME/focus/도움말접근 | CPU `1fe649add2558aca14b30785b7498fa4820986099e639d15ea5f716e86689f0e`; 이후service/authsnapshot전체IME입증은아님 |
| [Linux reader](cpu-native-recovery-validation.md) / [큰 report](cpu-report-stress-validation.md) |무음Orca처리/commit·cancel,1.9MB225pages/fullcopy/export와HWM70.8MiB | CPU `166946017de0b3d0589bf3d02bd864d004d90f3457ff2e314fe5e10a5f865417`; 모든화면·음성·3OS·현재binarypeak아님 |
| [Linux user-service terminal22](evidence/owned-child-terminal-native-20261004/root-review.json) |실제RGCLC·exit0/42/126·typed미확인·78PID/14cgroup부재 | startupalias/전체deadline후속이전immutable snapshot. 새로운samebootpreB계약이검증됐다는근거아님 |

위 manifest 비교는 저장된before/after의hash와 감사시디스크파일을 직접비교한 결과다. old root-review의 inputs_match_current는 **그 review 시점**의 current를 뜻한다. 새testsource 추가 및 platformcfg 차이까지 한 binary의 성공으로 합산하지 않는다.

### 감사시 소스 관측

관측 UTC: 2026-10-03T18:16:39.345562+00:00. 아래선택7파일을 hashlib.sha256으로읽었으며 전체workspacefreeze/빌드재현manifest가 아니다. root 진행중후속변경은 새snapshot으로 검증한다.

| 파일 | 감사시 SHA256 |
| --- | --- |
| `crates/engine/src/process.rs` | `04fd175181c244db0f9359d12e7914fec38ce30a4142093708c310710359cc38` |
| `crates/engine/src/process/guardian.rs` | `b37d41cb5f43bcbf644409ffcab6bd01195b1c2710be853ad845b3c9aa4029af` |
| `crates/engine/src/process/user_service.rs` | `ab8699d3b5a99930049d61e4864c8088eb946086001c8b691d4b5b85aa0c6040` |
| `crates/engine/src/process/windows.rs` | `51e9ba23a3adeaca53586eb63214d939b0a91ada7e3515e3fa3a9e04ebf8a091` |
| `crates/engine/src/cookies.rs` | `1ca837bae015a24311489711eaf3081988529881dc1020273a0bd1a6597c0146` |
| `crates/engine/src/cookies/windows.rs` | `96329a3b9d3a083635f0f700c604407741ac2682ab0fc791fb835bdb441f443a` |
| `crates/app/src/accessible.rs` | `04596fea9b87a054302aac68460584b07da5a6039126841169a3f53957ab7055` |

## R01~R13 P0 인수 표

| 요구사항 | 판정 | 완료된 scoped 부분 / 남은 인수 |
| --- | --- | --- |
| R01 | 부분 | Linuxprogress/ETA·볼륨·interface0/unknown구별·FFmpegfeed 기반검증. Windows/macOS 미지원카운터는측정불가표시인수 필요하며 모든OS에nativecounter구현을 새필수조건으로추가하지 않음 |
| R02 | 부분 | OSprovider/기본폴더·선택/저장/오류보존 구현. userDocuments를건드리지않는합성시험은있으나 실제한글profile/OneDrive리디렉션·권한거절인수 없음 |
| R03 | 부분 | Linux10k/windowed128page·watch·외부rename·잘못된root·후속CloseWait완료. Windowsnativeidentity CLI와emptylibraryGUI는있지만현재loadedrows/scan/OS조합은별도 |
| R04 | 부분 | actuallocal yt-dlp/FFmpeg synthetic HTTP403/shortresponse/resume,Unix/Windows sessionauth·safeprocess기반. realYouTube/치지직VOD·8hterminal·actualdisk/sleep·samebootcookie회수는미완료 |
| R05 | 부분 | ranges/비용설명/외부FFmpeg·pinyt-dlp管理installedGUI. 현재officialtoolchaincut정확도/version matrix·FFmpeg/Deno multi-component공급/notice/3OS설치남음 |
| R06 | 부분 | choice저장/복원·3theme사이징/150·200settings확인. 실제SystemOSevent/첫프레임·Windows/macOSmatrix남음 |
| R07 P0 | 부분 | typedquality/range/subtitle/advancedcatalog/정제plan·conflict/nojob/reuseoriginalsnapshotGUI. 모든노출옵션설명/예시/기본/충돌/version-or-unverified의현재catalog최종대조와officialpin조합인수남음 |
| R08 | 부분 | Linuxdefaultuser-service detachedcleanup·nativeWindowsJob·ownedChild/readerunwind/publicinterrupt·Linuxhiddenrestore·단기비용. samebootreceipt/현재manager회귀·OSsleep/crash·GUI장시간비용/최종UX남음 |
| R09 | P0 아님 | P2OAuth/upload미착수;현재P0차단목록에중복삽입하지 않음 |
| R10 | scoped 완료/운영부분 | AGENTS/docorder/workitem/최대3하위·공유담당/실제검증계약있음. Git정상저장소가없어commit/worktree는환경block;code/doc개발을막지않음 |
| R11 | 부분 | workflow3OS정의+actualLinux/WindowsGNU build/test/clippy. remotePR/Actions·macOS·설치/서명/upgrade·최소OS/ARM 및Windows배포toolchain정책남음. GNU인수와미구비MSVC조사를구분하며요구사항에없는MSVC추가필수조건을만들지않음 |
| R12 | P0 아님 | P1선택환경benchmark미착수;기본R01관측과혼동하지 않음 |
| R13 P0 | 부분 | offline reportstages/version/redaction/MDJSONpreview/copy/create-onlysave/job보존LinuxGUI. Windows/macOSclipboard·reader·currentfullflows 남음. P1knownissues/P2API는지금요구안함 |

S00→S07는아직어느단계도전체완료로승격하지않는다. S10 P0는S02이후/S07이전필수이며S08/S09순번때문에후순위로미루지않는다. S11updater는사용자가지정한 기능완성→Git/실제CI후속이라지금P0대체로착수하지않는다.

## 오래된 실패를 현재 차단으로 중복 집계하지 않은 항목

- 초기Linuxguardian SIGKILL detached생존은현재user-systemd/control-group보호와실제회귀가있다. 남은것은최신checkpoint/preBreceipt·OS고장·장시간조합이지과거direct실패전체를다시구현하는것이아니다.
- Linux200%settingsIME/footerclip은ScrollArea후6case통과했다. Windows/macOS및현재snapshot전체reader가미검증인것과분리한다.
- 원report전체galley HWM511.8MiB는8KiBpager후70.8MiB로감소했다. 정확2MiBunit경계를약1.9MBGUI와분리하되이미전체fullcopy/export를시험했다는사실은남긴다.
- LinuxlibraryCloseWait초기busyproxy실패는후속entry-state35ms/lockrelease후116ms종료로보완됐다. 과거raw를지우지않고Windowsloadedlibrary인수로확대하지않는다.
- WindowsfirstshowAccessKitpanic·matchedkeyboardharness이슈와raw183collision은후속수정/별도fixture범위로구분한다. privateclipboardstation183은아직다른실패이며해결되지않았다.
- publicstartupnoR15초전계약은현재150msEOF/5초정리actual4case로대체됐다. 짧은deadline을cleanup5초미포함반환으로오해하거나childreap를reader종료증거로바꾸지않는다.

## 바로 진행 가능 / 실제 제약

**지금 진행 가능:** samebootpreB/receipt·실패인수 구현, source동결후Linux전체검사와native22재실행, WindowsprivateGUIcapability 및samebinaryauth4mode, Linux현재필수화면keyboard/System/theme/소규모성능재검증,Windowsloadedlibrary/diagnosticpreview-save,합성officialFFmpegcut·archive/notice구성,운영체제독립catalog검수. 이들에추가실계정·실영상·외부메시지승인은필요하지않다.

**시간/소유권 대기:** 새8harchive의관측UTC 2026-10-03T18:16:00.336585+00:00은state=watching,download/guardian scope terminal은둘다null이다. 현재soaklaunch CPU는166946...이고 actualterminal/DBreopen/reap없이는PASS아님. 감사는archive만읽었으며livePID독립대조나프로세스종료를하지않았다. root가기존ownedfixture를계속관측하며새소스를그시험의실행소스로혼동하지않는다.

**환경block:** macOS호스트가없음. 실제WindowsTSF/reader·System이벤트를사용자desktop분리해서검증할환경은미확보. MSVC기존cl/link/SDK/targets는readiness관측에서없음. 사용자전용toolpayload가능성을검토할수있지만admin설치는별도승인이고GNU를실패로분류할이유는아님. Windowsprivateclipboardstation NULL/create-only는183으로실패했고기존station재사용·WinSta0fallback을하지않음. 별도review된OS시험환경이필요하다.

**사용자범위/결정block:** 사용자가실제영상대신합성fixture시험을지시했으므로realYouTube/치지직영상다운로드를이번감사나자동개발로임의수행하지않는다. 실제계정/쿠키인증·외부전송은별도승인. 실제redirectedDocuments/권한실패는owned별도profile환경없이는userDocuments를변경하지않는다. 정상Git/remote주소·배포채널/서명·최소OS/ARM정책은불명확하며기존.git재구성/forcepush/외부공개/유료리소스로우회하지않는다. unsigned패키지·CI설정준비등독립작업은계속가능하다.

## 실제 감사 명령과 한계

rg로요구사항·ROADMAP·P0-status·ADR·관련workitem/validation/source를읽고Pythonjson/hashlib로위result·root-review·before/aftermanifest와current7fileSHA를대조했다. Windows/GUI/Cargo/실제사이트를새로실행하지않았으며외부browse도없다. `python3 scripts/check_docs.py`는379개문서/로컬링크검사PASS였다. 이문서는2026-10-04checkpoint이며변하는archive/root후속코드의완료상태를자동반영하지않는다.
