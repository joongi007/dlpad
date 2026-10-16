# Windows FFmpeg 다음 P0 수직 인수 — 읽기 감사

상태: 준비, 실제 실행 없음. 프로젝트·네트워크·native 변경 없음. 현재 FFmpeg 공급은 Proposed이며 자동 설치 비활성 정책을 유지한다. 이 시험은 기존 확보한 원 executable을 임시 합성 미디어에 사용하는 허용된 개발 검증이다.

## 확보된 원본과 범위

원 root `.local-tools/ffmpeg-win64-inventory-f5uj2z2m`. 고정 ZIP `ffmpeg-n9.0.2-17-g2a571b6068-win64-lgpl-9.0.zip` 171535354B/SHA256 `6b264b9e6019103f601d98c292bd332fd87acf1c5e941ddff4fb71760fe63432`. 원 manifest의 component는 ffmpeg.exe134497280B/SHA `0e482b853f2fd4821055d20cdce510f0eea6027037c04b994f454a2136e8868e`, ffprobe.exe134296064B/SHA `b9627185ffbad1d4fc6ef953868ca9ad8df75fc8dd035fcab58aca78caa2d23e`, LICENSE.txt7651B/SHA `da7eabb7bafdf7d3ae5e9f223aa5bdc1eece45ac569dc21b3b037520b4464768`. 본 감사는 이 선언을 읽었고 대형원파일 hash를 재측정하지 않았다.

S04-windows-ffmpeg-native-version.md 실제 두 version은 `n9.0.2-17-g2a571b6068-20260930`, exit0·원size/SHA보존·private/input불변이었다. 당시 ffprobe operation workspace동시변경 source snapshot FAIL도 보존되어 있으며 toolruntime 성공과 source전체검증을 합치지 않는다. 실제 mappedZIP 설치·선택은 S04-windows-ffmpeg-managed-install.md와 별도 근거다. 이전 Unix local_media.rs 실제빠른/정확cut+decode는 Windows 인수를 대체하지 않는다.

LICENSE.txt만으로 공급 완료가 아니다. S04-windows-ffmpeg-supply-adoption.md가 고정한 Windows enabled recipe/fullsource/transitive notice/build image·runtime dependency/attestation gaps, 공개배포 source제공과noticebundle는 그대로 남는다. 보유 source자료를 법적 완비 판정으로 바꾸지 않는다.

## 가장 작은 다음 vertical: 실제 Windows 1초 구간 저장·decode

fresh Windows native example/test와 기존 private CLI host를 사용한다. root가 fresh dlpad-process-guard를 먼저 build/register하고 helperSHA+bytes, fixtureSHA+bytes, 원ffmpeg/ffprobe/version/licenseSHA를 manifest로 고정한다. 원executable HANDLE/nativeidentity+share read pins를 실행 전후 확인한다. 표준 argv만 사용하며 shell/사용자영상/계정/외부site는 없다.

1. 원 TEMP의 fresh ownedroot/media 아래 2초160x90/30fps 합성 source를 생성한다. 실제 argv 후보 `-nostdin -hide_banner -loglevel error -f lavfi -i color=c=blue:s=160x90:r=30 -t 2 -c:v mpeg4 -g 15 <source.mp4>`이며 원 ProcessSpec deadline20s/log상한을 적용한다. silent video면 audiocodec 의존성을 먼저 늘리지 않는다.
2. Windows public DownloadRepository 경로를 인수하려면 기존 Unix local_media의 loopback media transport를 Windows용 owned127.0.0.1 ephemeral helper로 이식한다. 서버는 합성단일source만 제공하고 ownedprocess 원HANDLE/timeout/EOF 정리를 가진다. 제품용 webUI로 확장하지 않는다. 공식원 yt-dlp standalone도 source/helperSHA/version을 고정한다. shell실행금지. 직접ffmpeg cut만으로 Repository완료라고 쓰지 않는다.
3. 외부 ffmpeg는 PATH밖 `한글 도구 폴더/ffmpeg.exe`로 TEMP ownedcopy를 만든다. DownloadOptions.ffmpeg absolute literal path, output `한글 저장 폴더`, time_range 0.5..1.5(빠른 GOP경계 비교), exact_cut false/true를 구분한다. core현재 plan은 --ffmpeg-location 한arg, --download-sections 별도typedarg, exact일때만 --force-keyframes-at-cuts이다. 추가 exact off-keyframe 0.35..1.35는 frameprecision 확인으로 분리 가능하다.
4. 각각 actual artifact를 ffprobe `-v error -show_entries format=duration:stream=codec_name,width,height -of json <artifact>` boundedJSON으로 검사하고 ffmpeg `-nostdin -v error -i <artifact> -f null -` 실제decode exit0 확인한다. exact1초는 frame단위 tolerance(30fps 기준 1frame+container rounding)와 native actual값을 명시한다. fast는 seek/GOP 특성 때문에 exactprecision 동일요구를 하지 않지만 nonempty/실제decode/대략구간·명시durationbound는 검사한다. 정확구간을 fast결과로 대체하지 않는다.
5. SQLite Attempt의 실제 FFmpeg경로/version, Downloaded/artifactnativeidentity/실제크기·SHA/DB reopen을 검사한다. 원source/video/toolcopy/license/foreignsentinel SHA 전후불변, 단계별 실제 argv·elapsed·originalprocesscreation·terminalreceipt/Job0를 boundedraw로 보존한다. 관리FFmpeg를 사용할 경우 기존 publicexecutionlease를 실제 reader단계에 연결하고 receipt/finish/reuse를 확인한다. 단순 외부copy시험은 관리guardian lifetime완료로 확대하지 않는다.

## 실패 인수

원형식unknown version, nonexistent/executable directory, toolversion中cancel, 실제cut中cancel과shortdeadline을 각 구분한다. tool가매우빠르면 sleep시간만으로 취소성공을 주장하지 않고 actualstarted progress/ownedcontrol ready를 기다린 뒤 취소한다. timeout은 실제toolphase를확인할 여유가 있는longfinite 합성lavfi를 별도사용한다. Interrupted/TimedOuttyped결과·artifact publication0/원pendingreceipt정책·원외부source/tool삭제0를검사한다. 실패사진/로그 보존, timeout이전spawn도달못한case를cuttimeoutPASS로취급하지않는다.

## root 파일 분담 제안

새 Windows actual media fixture example와 전용nativecollector를 추가하고 기존 options/download 공유코드는 read-only 우선한다. local_media.rs Unix-only를 이름만Windows시험으로바꾸지 않는다. root는 Cargo/helper/sourcefreeze/공유 ProcessSpec와 RepositoryAPI통합; 독립fixture담당은 합성미디어+boundedtransport+DB/decode인수; supply담당은 고정source/notice후속을 분리한다. 실제GUI구간옵션·site·최소OS·공급채택은 이번 CLI mediavertical외 별도gate다.
