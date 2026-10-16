# S04: 고정 FFmpeg TAR/XZ native 검증 example

- 상태: in-progress
- 분류: test
- 우선순위: P0
- 요구사항: R03, R05, R11
- 담당/수정 범위: metrics_recovery — examples/verify_ffmpeg_tar_xz.rs 및 이 문서만
- 선행 작업: root bounded TAR/XZ 공개 API 및 기존 고정 Linux 공급 파일

## 사용자 결과

원 source read-only File에서 고정 root/mappings의 ffmpeg·ffprobe·LICENSE 세 component를 각각 익명 tempfile에 bounded 추출하고 exactsize/SHA와 원 source SHA 불변을 확인한다. 도구 실행·설치·active DB·공급 채택은 하지 않는다.

## 계약과 인수

CLI 입력은 절대 sourcePath 한 개이며 parent/symlink/reparse/nonregular를 거절한다. 원 archive137863452B/SHA와 기존감사의 세component size/hash를 상수로 고정했다. TarLimits entries4096/total768MiB/selected384MiB/stream800MiB 및 전체180초 같은 절대deadline을 사용한다. 세component 각각 원 compressed source부터 전체 stream을 검증하므로 반복decode 비용을 숨기지 않는다. 익명 output sync/seek/streamSHA를 별도로 대조하고 drop으로 ownedtemp만 닫는다. 전체source 최종hash를 다시 확인하고 component별시간/bytes/hash 및 sourcebefore/after boundedJSON4096을 출력한다. failure는 고정 redactedJSON/exit2다.

## 검증과 handoff

구현과 담당파일 개별rustfmt만 수행했다. Cargo/releasebuild/actualarchive 측정은 root가 수행한다. 실제 성공·RSS·처리시간은 아직 주장하지 않는다. 새로운 다운로드·재패킹·실행·설치·공급채택·기존파일 삭제는 없다. 원 archive path는 출력하지 않는다.

## 공식 원 archive 실제 native 추출

root는release예제 SHA dd0b53a0768b33b32b3e5fa62c7bc8ddb68020f3197b5c2ae43e063d4b633ada/486280byte를실제로실행했다. 고정archive에서ffmpeg142534568B/ffprobe142305128B/LICENSE7651B각정확SHA를확인하고전후source SHA가같았다. 전체73member를판독하는3회stream은모두PASS이며툴실행·설치·활성화·재포장은없다. [원시 결과](../validation/evidence/ffmpeg-native-tar-xz-first/result.json). GNU/TAR전체통과를처음실증했으며이제관리공급채택이완료됐다는뜻은아니다.

최대RSS(/usr/bin/time)는10348KiB,wall32.13초이고제품Instant elapsed35073ms/외부Pythonelapsed35.11초를각각원시보존한다. 서로다른시간관측을하나의정밀값으로혼합하지않는다. component별대략10.6–11.2초이며각선택마다전체archive를다시판독해3회순회한다. 이486280B는독립검증예제전체크기이며UI앱증분크기나FFmpeg크기가아니다. 실사용installer연결의총임시용량·단일순회최적화·실제정확버전선택·trust/notice/3OS/fullworkspace는남는다.

강화된XZ5회귀(과대dictionaryOutOfMemory actualassert 포함)와engineall-targetClippy가terminalexit0이다. TempDir는원File/output종료뒤정리되며Windowsopenhandle정리문제를피한다. 최신WindowsGNU전체collector는실행중이며terminal전에는통과로집계하지않는다.
