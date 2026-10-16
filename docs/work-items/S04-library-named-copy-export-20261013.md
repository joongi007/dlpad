# S04: 파일 목록의 이름 지정 복사 내보내기

- 상태: in-progress
- 분류: feature
- 우선순위: P1
- 요구사항: R02, R08, R11
- 담당/수정 범위: installer_batch — 새 engine file_export.rs 후보; root — lib/schema/library UI 통합
- 선행 작업: 파일 목록 선택·신원 재검증, 제목 naming 정책

## 사용자 결과

원본 영상은 그대로 유지하며 원하는 이름으로 새 파일을 복사해 내보낸다. 제목/파일명을 알아보기 쉽게 관리한다.

## 계약과 범위

create-only destination, 원본·출력부모 native identity 고정, bounded chunk/progress/cancel/timeout, 실패 시 기존 파일은 보존. 기존 user data overwrite/delete 금지. Cargo와 공용 interface/등록은 root 조정; 독립 후보만 편집한다.

## 인수 조건

Windows 금지/예약 이름·한글·emoji·길이·동명충돌·source/parent 교체·쓰기실패·취소·부분출력·복구를 합성 임시 fixture로 검증한다. stream 전체를 메모리에 적재하지 않고 완료 검증 후 공개한다.

## 검증과 handoff

2026-10-14: 격리 소스에 created-relative-file capability, file_export 및 Library UI/기존 picker/단일 worker를 통합했다. feedback-copy-core-v1의 Windows cap7+export8 실제15/0은 Root 독립 대조를 통과했다. feedback-copy-ui-v1의 실제 context/picker/Stop 회귀와 Library43/Main53를 포함한 합계101/0(3 ignored)은 원로그로 보존한다. source·stage·source 재확인·최종본의 네 번 해시 읽기, 64KiB chunk, 1TiB/300초 협조적 한계, 진행 mailbox1을 사용한다. NT 공개 이후 불확실한 실패에는 경로/identity 사실만 보존하며 삭제 권한으로 사용하지 않는다. 동일 이름의 기존 파일은 덮어쓰지 않는다. background footer와 1회 완료 알림을 feedback-copy-background-v1에서 추가 검증했다. 제품/설치본에는 아직 미통합이며 실제 private copy UI/전체 회귀/비용 측정은 남아 있다.

2026-10-14 현재 통합 상태: 후보는 제품 source669/v22 설치본에 포함되었다. Windows전체네명령1,524/0/42와73화면/정상·긴경로설치 Root검토를 통과했다. 앞 문단의 미통합 표현은 당시 격리 후보의 기록이다. 제목다운로드·이름지정복사 공개API 실제실행은 [별도 합성시험](S02-windows-v22-title-copy-api-smoke-20261014.md)에서계속검증한다. 물리picker/실영상/늦은취소·전체사용자인수는별도다.
