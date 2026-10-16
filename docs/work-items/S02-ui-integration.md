# S02/S10: 네이티브 개발 UI 통합

- 상태: review
- 분류: feature
- 우선순위: P0
- 요구사항: R01, R03, R04, R05, R06, R07, R08, R13
- 담당/수정 범위: 주 담당; crates/app/src/main.rs/catalog.rs, module exports, CI, 통합 문서
- 선행 작업: S00 실험, 독립 core/engine foundation. S00 출시 gate는 미충족

## 사용자 결과

URL·구간·품질·기존 tool 경로를 입력하고 단일 worker 시작/취소, 실패 자료 미리보기·복사·새 파일 저장을 사용한다. 로컬 MPD는 구조만 첨부하며 로그 수정/제외를 재정제한다. 임시 파일 목록이 이벤트에 맞춰 갱신되고 대상 볼륨 여유를 확인한다.

## 계약과 범위

main은 workspace/schema·통합을 소유한다. 진단, domain/process/download, library/tools/instance는 각각 다른 담당이 별도 파일로 준비했다. 공유 Git가 정상 저장소가 아니므로 .git 보존과 파일 분리로 진행했다. report export는 create-new이고 외부 전송하지 않는다. user Documents·실제 영상은 시험하지 않는다. Rust 네이티브로만 UI를 구성한다.

## 인수 조건

합성 fixture와 temporary root에서 네이티브 창, DB rollback·실패 상태·취소·artifact 소유 검증·진단 secret 제거를 검증한다. 클릭/clipboard/IME/3 OS 실제 동작을 unit test로 완료 표시하지 않는다. 전체 P0/출시 gate는 P0-status에 남긴다.

## 검증과 handoff

[측정](../validation/S00.md), [요구사항 상태](../validation/P0-status.md)를 따른다. 마지막 통합 명령과 결과는 validation 문서에 기록한다. synthetic downloader와 snapshot/Attempt/크래시 간격 복구 테스트를 주 담당이 다시 실행한다. 실제 yt-dlp/FFmpeg와 장시간 작업은 미검증이다.
