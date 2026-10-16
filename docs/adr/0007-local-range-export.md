# 0007: Windows 로컬 구간 저장의 게시와 정리 사실

- 상태: Proposed — 개발 후보 구현과 scoped 실험, 최종 GUI 인수 전
- 날짜: 2026-10-11
- 요구사항: R04, R05, R08

## 문제

로컬 파일의 시각 선택을 새 구간 파일로 저장한다. 취소·worker 종료·관리 도구 scope 오류가 이미 생성한 파일을 없었던 것으로 표시하면 안 된다. 경로 문자열은 기존 사용자 파일의 삭제 권한이 아니다.

## 후보 결정

원 Prepared source pin을 유지하고 기존 상위 폴더의 원 HANDLE 아래 새 전용 directory/stage를 FILE_CREATE로 만든다. FFmpeg argv는 입력seek와 accurate_seek, 출력 duration 및 file protocol만 사용한다. 공급 LGPL FFmpeg에 존재하지 않는 libx264를 가정하지 않고 내장 MPEG-4 video(q:v3)와 AAC로 개발 결과를 만든다. 외부 GPL encoder 공급이나 제품 Rust dependency를 추가하지 않는다.

stage의 원 read cap은 FFmpeg writer와 호환하되 DELETE를 허용하지 않는다. 도구 terminal 뒤 read-only validation cap이 신규 쓰기/삭제를 차단하고 ffprobe 길이·단일 RGB frame을 확인한다. 최종 clip.mp4는 create-only hardlink로 게시하고 stage는 같은 bytes를 가리키는 링크로 보존한다. 원본·기존 파일·partial 자동 삭제는 없다. source/root/nativeID와 현재 결과 검증은 역사적 ownership·전체 frame 완전성·다른 플레이어 호환성을 뜻하지 않는다.

ExportCompletion은 파일 결과와 cleanup_uncertain를 구분한다. finish 실패·panic 뒤 생성 사실을 버리지 않는다. worker stop ACK와 process cleanup은 별도다. 원cap을 오류 DTO에 저장하거나 경로 사실로 원cap을 재구성하지 않는다. 미확인 종료는 새 실행·정상 종료를 차단한다.

## 비용과 제한

요청/완료 채널은 각각1, export worker는 활성 요청 동안 thread1이다. 인코딩 thread1, 전체180초/encode120초, 구간10분/결과256MiB 개발 상한을 화면에 안내한다. FFmpeg -fs는 엄격한 disk hard cap이 아니며 측정한 초과 파일도 보존한다. 단일 allocation 상한은 process RSS 상한이 아니다. 긴 소스 전체를 Rust 메모리에 읽지 않는다. 원cap 회귀 및 실제2초 red/blue의 export 길이·blue frame·nativeID·원 byte 보존은 통과했으며 actual GUI/긴 소스·inflight취소 인수는 남는다.

## 검증

[작업 기록](../work-items/S05-local-range-export-20261011.md)의 새 Windows proof를 따른다. GUI와 명시 취소·닫기·도구 lease 실패를 통합 검증한 뒤 scoped 채택 여부를 갱신한다. 다른 OS/정식 updater·설치 trust는 이 결정으로 완료하지 않는다.

2026-10-11 추가 실험: 8시간 실제 encoded synthetic source의 후반 export에서 기본 encoder frame timebase가2fps timestamp를 반올림하여 1초로 짧아짐을 관측했다. millisecond encoder timebase와 passthrough로 원 common timeline을 보존하여1.5초/3frames, 처리0.263초와원SHA 유지 회귀가통과했다. 독립video setpts reset은 A/V phase를 지우므로 채택하지 않는다. 30fps 강제는3→45frames로비용이증가해채택하지않는다. 짧은 실제 flash/tone aligned/200msdelay 네 경우의 common epoch·원본 보존 회귀를 추가했다. native GUI/사용자 인수 gate는 별도다.
