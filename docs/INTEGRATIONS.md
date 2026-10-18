# 도구·고급 옵션·인증

## 도구 수명

실행 경로 선택: 작업/사용자 명시 경로 → 설정에서 선택한 관리 설치 → PATH에서 발견한 후보. 발견만으로 실행하지 않고 경로와 버전을 확인한다. system 도구를 앱이 업데이트하거나 삭제하지 않는다.

manifest 필드: tool ID, version, target, source URL, digest, install root, installed_at, ownership, component list, license reference, active users. FFmpeg와 ffprobe는 같은 배포 세트로 추적한다. yt-dlp standalone과 JS runtime 크기도 설치 합계에 포함한다.

관리 설치: 검증된 배포 manifest → 임시 다운로드 → digest/가능한 서명 검증 → 경로 탈출 없는 압축 해제 → 실행 가능·버전 검사 → 원자적 활성화. 실패 시 기존 버전 유지. 업데이트도 같은 경로이며 실행 중 버전은 pin한다. 앱이 관리하는 root 내부인지 확인한 파일만 삭제한다. 최신 URL을 매 실행 추적하거나 yt-dlp 자체 updater가 관리 manifest를 우회하게 하지 않는다.

FFmpeg는 공식 사이트가 소스를 제공하고 플랫폼 빌드는 외부 공급처를 안내한다. 배포 공급처·빌드 옵션·라이선스·해시를 선정한 다음에만 자동 설치를 활성화한다. 서명/공급처 결정 없이 자동 설치 코드를 완성됐다고 취급하지 않는다. [FFmpeg 다운로드](https://ffmpeg.org/download.html).

YouTube 지원에는 외부 JS runtime과 challenge solver 구성 확인이 필요하다. UI 웹 런타임과 별개이며 `YouTube 지원 구성요소`로 크기와 용도를 설명한다. Deno 등을 후보로 검증하고 버전을 관리한다. 원격 실행 코드를 매번 무검증으로 가져오지 않는다. [yt-dlp EJS](https://github.com/yt-dlp/yt-dlp/wiki/EJS).

## 2026-10-01 공급 증거와 설치 gate

[공급 검증 기록](validation/tool-supply-evidence.md)에 tag·정확한 다운로드 크기·checksum·license·플랫폼 한계를 고정했다. yt-dlp stable 2026.08.19의 공개 checksum은 격리 GPG keyring에서 예상 fingerprint 서명과 합성 변조 거절을 검증했다. executable bytes 검증·설치·3 OS 실행은 이 결과에 포함하지 않는다.

FFmpeg BtbN 날짜 고정 LGPL build와 Deno v2.9.7은 공급 **후보**다. archive metadata/checksum을 확인했으며 archive/실행 파일·서명/attestation·압축 해제·license/source/notice·codec/solver·전체 크기 검증은 남는다. FFmpeg 공급처를 사용자 확정으로 표현하지 않는다. 기존 단일 executable manifest의 sha256에 archive digest를 넣지 않으며 ffmpeg/ffprobe 배포 세트와 component별 digest 계약을 먼저 완성한다.

현재 [EJS guide](https://github.com/yt-dlp/yt-dlp/wiki/EJS)는 Deno >=2.3 또는 Node >=22를 요구한다. 공식 standalone의 bundled solver를 사용하고 실행 시 원격 component 자동 갱신을 추가하지 않는다. Deno Windows 최소 version 1709 안내는 [공식 설치 문서](https://docs.deno.com/runtime/getting_started/installation/)에서 확인했지만 dlpad의 OS 지원/실증을 대신하지 않는다. 기존 외부 FFmpeg 4.4.2의 Linux version/license 관측은 새로운 관리 공급 후보 검증과 구분한다.

## 고급 옵션 계약

앱은 재현 가능한 실행을 위해 외부 사용자 config를 자동 상속하지 않는다. 설치 버전에서 검증한 yt-dlp config 차단 옵션과 plugin 탐색 정책을 적용한다. config 가져오기는 파일 내용을 파싱·검증·미리보기한 후 앱 프로필로 저장한다. shell 스크립트가 아니다.

우선순위: 기본 preset < 전역 프로필 < 작업별 설정. UI와 advanced 옵션의 동일 키 충돌은 조용히 덮지 않고 어느 값이 적용되는지 표시한다. 프로세스 제어·출력 framing·작업 경로 같은 앱 예약 옵션은 마지막 덮어쓰기 대신 입력 시 차단한다.

| 분류 | 예 | 정책 |
|---|---|---|
| 설명 있는 주요 옵션 | 품질, 자막, 속도 제한, 재시도, fragment 동시성 | 타입 기반 입력, 범위 검사 |
| 추가 yt-dlp 옵션 | extractor args, format expression | 설치 버전 검증·비밀값 마스킹·도움말 |
| 앱 예약 | 출력 템플릿/경로, progress/print, exec, tool 경로, config/plugin 로딩, updater | 작업 계약을 깨는 raw override 차단; 대응하는 관리 UI 사용 |
| FFmpeg 옵션 | codec, CRF, preset, audio bitrate | 입력/출력/후처리 위치를 구분한 argv 배열 |
| 미지원/미확인 | 새로운 옵션이나 build별 codec | `검증되지 않음`으로 표시하고 오류를 설명; 임의 설명 생성 금지 |

`--exec` 등 임의 명령 실행과 plugin 로딩은 일반 config 가져오기로 활성화하지 않는다. 별도 trusted extension 모드를 만들려면 ADR이 필요하다. URL·경로·옵션은 shell expansion하지 않는다. `--`, quote, Unicode, Windows 경로, 중복 key를 parser 테스트에 포함한다.

옵션 카탈로그 스키마: id, tool, since/until, value_type, default, Korean label, summary, example, tradeoff, conflicts, sensitive, upstream_help. 모든 UI 노출 항목은 설명을 필수로 갖는다. 버전별 `--help` snapshot을 비교해 변경을 찾는다. 전체 FFmpeg 옵션의 영구 수동 번역은 약속하지 않고 알려진 카탈로그 + 해당 설치의 원문 도움말 + 미확인 표시로 범위를 정직하게 유지한다.

실행 계약 후보는 yt-dlp JSON metadata/progress template와 FFmpeg `-progress`다. 정확한 argv는 고정 버전 fixture로 검증 후 구현한다. 미디어 병합·재인코딩·seek의 시간 비용을 서로 구분한다. [yt-dlp 사용·설정](https://github.com/yt-dlp/yt-dlp#usage-and-options), [FFmpeg 옵션](https://ffmpeg.org/ffmpeg.html).

## 쿠키와 비밀

브라우저 프로필 선택이나 Netscape cookie 파일 가져오기를 제공한다. 모든 브라우저/OS에서 복호화가 가능하다고 약속하지 않는다. 개인 브라우저 profile 전체를 복제하지 않는다. 긴 작업 복구는 브라우저 캐시 대신 partial 파일을 사용한다.

필요한 임시 쿠키는 사용자 전용 권한으로 만들고 작업 종료·다음 시작 시 청소한다. 평문 영구 저장은 기본 사용하지 않는다. 로그의 URL query, Authorization, cookie, proxy password, OAuth code, session URI를 마스킹하고 진단 내보내기에도 적용한다. 비밀 저장소가 없으면 세션 한정 사용을 제안한다.

## 실패 진단과 AI 연결

실패 로그와 MPD의 진단 내보내기, 알려진 이슈 조회, AI 제공자 연결은 [실패 진단 계약](DIAGNOSTICS.md)을 따른다. AI 분석 계정은 YouTube 업로드 OAuth와 별개이며 기본 다운로드에 필요하지 않다.

## YouTube 업로드 (P2)

시스템 브라우저의 desktop OAuth + PKCE/state + loopback redirect를 사용한다. 업로드 scope를 최소화하고 refresh token은 OS secret store에 저장한다. 이 loopback은 인증 콜백에만 잠깐 사용하며 UI 서버가 아니다. 계정 연결 해제는 로컬 비밀 삭제와 가능한 원격 철회를 수행한다. [Desktop OAuth](https://developers.google.com/identity/protocols/oauth2/native-app).

기본 privacy는 private. 연결 후 다운로드 화면에 `완료 후 YouTube 업로드`가 활성화되며 자동 체크하지 않는다. 제목·설명·카테고리·아동용 여부·private/unlisted/public을 작업마다 확정한다. 계정과 공개 범위를 시작 전에 요약한다.

Resumable upload session과 서버가 확인한 offset을 저장하고 청크 단위로 전송한다. 재시작 시 서버 offset을 조회한 후 재개한다. 완료 응답을 놓쳐 성공 여부가 불명확하면 NeedsReview로 두고 새 insert를 무작정 재전송하지 않는다. 공개 후 삭제는 로컬 작업 취소와 별도 동작이다. [재개 프로토콜](https://developers.google.com/youtube/v3/guides/using_resumable_upload_protocol).

검증되지 않은 API 프로젝트의 업로드는 비공개 제한을 받을 수 있다. 따라서 공개 선택 UI는 실제 프로젝트 상태를 반영한다. API 쿼터·채널별 길이/크기 한도·계정 상태는 구현 시 재확인하고 오류를 안내한다. OAuth 동의화면 검증과 YouTube API 감사는 별개 준비 사항이다. [videos.insert](https://developers.google.com/youtube/v3/docs/videos/insert).

## 개발 UI의 현재 연결

고급 옵션은 속도 제한(B/s), 재시도(0~20), 조각 동시성(1~16)만 노출한다. 빈 값은 기본값, 불법 값은 입력 옆 오류와 시작 거절이며 retry override는 argv에 한 번만 남는다. 설명·예·기본값·비용·충돌·검증되지 않은 버전 표시를 카탈로그에서 읽는다. 자막은 저장 안 함/한국어/영어/한·영과 별도 자동 생성 허용을 typed 옵션으로 연결했다. 구간과 함께 선택하면 시작을 거절한다. after_video 출력은 id·한/영 filepath/ext만 받아 source/staging 검증 뒤 영상과 함께 공개한다. config 가져오기/추가 옵션은 아직 연결하지 않았다. 정확 자르기는 명시 선택하며 전체 영상에는 cut 옵션을 넣지 않는다.

외부 FFmpeg 선택은 실행 파일의 절대 Unicode 경로만 받는다. 경로는 snapshot에 포함하며 canonical 파일의 -version을 확인해 Attempt별 기록과 실패 진단에 전달한다. 빈 선택은 기존 yt-dlp PATH 탐색이고 미확인 버전을 수집했다고 표시하지 않는다. 선택 파일/권한/버전 오류는 도구 확인 단계다. 관리 자동 설치의 공급처 결정은 별도 게이트다.

## 세션 쿠키 개발 연결 범위

CPU/accessible native 후보에 이번 요청의 Netscape 파일 경로 입력을 연결했다. 원본을 변경하지 않고worker에서0700/0600전용복사·최대1MiB/4096entry형식검증을 수행한다. 입력 경로는 worker로 전달한 뒤 비우고 설정에 저장하지 않는다. 버전조회에는쿠키를넣지않고metadata/download에만별도lease를전달한다. 인증raw로그/자유metadata는생략하고같은옵션snapshot으로재개할때도새쿠키를선택한다.

기본 UI/최종 스택 확정이나 R04 전체완료는 아니다. nonUnix는ACL검증없이지원한다고주장하지않고거절한다. 종료미확인typed오류는복사본을보존하며FD를닫는다. terminalownership확인·startup회수·실제GUI인수는 [작업 기록](work-items/S02-session-cookie-download-integration.md)에남긴게이트다.

## 2026-10-08 Windows 개발 후보 설치 연결

앱의 도구 화면에서 yt-dlp 직접 설치와 Deno2.9.7 v3 pinned notice·FFmpeg/ffprobe v2 mapped ZIP 공급을 선택할 수 있도록 연결했다. [Deno 배포](https://github.com/denoland/deno/releases/tag/v2.9.7), [BtbN 배포](https://github.com/BtbN/FFmpeg-Builds/releases/tag/autobuild-2026-09-30-13-08) 고정 후보와 기존 공급 inventory의 exact size/hash/components를 사용한다. 공개 배포·attestation·transitive notice/대응 source 완비는 이 연결로 채택하지 않는다. archive 공급 성공은inactive이며 앱에서 실제 version 검증 후 활성화·사용한다.

개발 installer는 기본 unchecked 선택3개를 제공한다. 원 앱 bundle prepare/검증 성공 뒤 같은worker/cancel/전체absolute300초안에서 도구별 최대120초로 공급한다. 도구실패는앱성공과별도표시하고기존active/성공도구를보존하며다음도구는미시작한다. appbundle/source/journal과projected canonical toolroot가겹치면새mutation전에거절한다. 기본userdata는앱과동일이며명시 `--tool-data` 개발프로필은TEMP격리시험용으로사용한다. 선택none은toolroot를만들지않는다.

앱 재실행은 기존 원 복구 worker에서 요청된 emptyfield의existingactive만 다시 whole-package/version 검증해typedpath로복원한다. 없는root·비활성후보·외부custompath를자동채택하거나다운로드하지않는다. 최종현재field가바뀌거나CloseWait이면원값을보존한다. 실패는복구오류로표시한다. 자세한 인수는 [작업](work-items/S03-windows-in-app-tool-install.md)에 기록한다.

## 공개 영상 미리보기의 도구 경로 (2026-10-08)

무음 preview는 단일 video-only 소스를 포함하는 낮은 해상도 형식을 선택한다. 선택한 Deno는 명시 argv와 Prepare 중 executable pin에 포함하며 외부 config/plugin 자동 상속·cookie 전달·remote component 자동 갱신은 추가하지 않는다. worker 도구 경로는 준비 시 snapshot이며 변경 적용에는 새 영상 확인과 이전 worker stop ACK가 필요하다. `runtime_environment=None` 경로를 managed execution lease/전용 Deno cache 완성으로 표현하지 않는다. 실제 공개 URL의 metadata·짧은 preview와 전체 다운로드·전체 사이트 호환은 별도 인수다. [시험 범위](work-items/S00-public-video-ui-walkthrough.md), [미리보기 결정](adr/0007-bounded-media-preview.md).

## Windows GUI의 수동 경로 설정 (2026-10-11 통합 중)

다운로드 환경의 세 경로를 수정하면 입력한 경로 기억으로 전환하며 도구 선택 저장으로 세 선택을 함께 저장한다. theme.sqlite의 별도 manual_tool_preferences 설정 key를 사용하고 schema 변경은 없다. 조회/저장은 UI thread가 아닌 lazy worker에서 수행한다. 경로≤32,768 UTF-8 bytes·JSON≤128KiB·고정3선택을 제한하고 빈 Manual·상대 경로·제어 문자·상위 경로 구성 요소를 거절한다. 잘못되거나 초과한 기존 설정은 자동 수리/덮어쓰기하지 않는다.

시작 DLPAD_YTDLP/DLPAD_FFMPEG/DLPAD_DENO 환경값은 빈값도 저장된 수동 경로보다 우선한다. 그 뒤 저장된 Manual, 검증된 managed 활성 경로를 적용하며 사용자 편집의 revision을 보존한다. 명시적 managed 사용은 선택한 slot만 Automatic으로 저장하고 다른 slot의 미저장 draft를 함께 저장하지 않는다. Automatic 선택 후 저장은 현재 입력을 검증된 managed 경로나 빈값으로 바꾼다. UI yt-dlp PATH 연결은 아직 미구현이며 문서 앞부분의 일반 발견 목표를 GUI 완료로 해석하지 않는다.

worker는 요청 channel1/result1/최신 pending1로 제한한다. 종료 중 active Store upsert가 끝나기 전에 Saved/ACK를 주장하지 않으며 unsent 최신 요청을 버린다. 원본 도구 파일을 저장 worker가 열거나 실행·복사·삭제하지 않는다. Cargo/feature/lock 의존성은 추가하지 않았다. 실제 Windows 저장 실패/locked upsert/재조회 회귀는 통과했고 실제 GUI 저장·재실행은 [진행 중](work-items/S04-manual-tool-preferences-20261011.md)이다.


## 앱 업데이트와 외부 도구 업데이트

앱 업데이트는 yt-dlp/FFmpeg/Deno 관리와 별개다. Windows CPU 앱0.1.1은 저장소에 고정한 Ed25519 운영 공개키로 원 metadata bytes를 검증하고 공식 GitHub HTTPS 공급만 사용한다. 공개 릴리스 조회나 내려받은 public-key 파일은 실행 신뢰가 아니다. 쿠키·인증·임의 proxy·사용자 URL override를 공급에 넣지 않는다. 서명·버전·단조 sequence·채널·만료·원 ZIP 크기/해시/멤버를 확인한 뒤 별도 새 bundle을 준비한다.

[서명 운영](UPDATE-SIGNING.md)의 로컬 DPAPI 키는 외부로 전송하지 않았다. CI의 비밀키 자동 공급은 별도이며 미설정 시 unsigned preview를 앱이 자동 실행하지 않는다. 최초 v0.1.0→업데이트 지원 버전은 수동 설치한다. 이후에도 고정 부모 자체의 키·epoch·보안 패치는 수동 설치 대상이다. 사용자 미디어·외부 설치 도구를 업데이트 cache처럼 정리하지 않는다.
