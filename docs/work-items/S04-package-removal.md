# S04: 비활성 managed package 원 파일 제거

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R03, R11
- 담당/수정 범위: metrics_recovery — tools/package_removal.rs 및 이 문서만
- 선행 작업: root verify_package/native_files/module/remove branch 연결

## 사용자 결과

활성 버전은 거절하고 전체 원 package의 구성원·manifest 검증과 native 파일 권한 preflight를 완료한 뒤에만 비활성 파일을 제거한다. foreign 항목이나 변경된 파일이 있으면 사전 실패에서는 아무 파일도 삭제하지 않는다.

## 계약과 범위

remove_package는 전체 verify_package를 먼저 실행한다. 원 directory pin과 최대10개 exact allowlist, component와 두 metadata의 원 FileId를 보유하고 모든 component pin에서 크기·SHA를 다시 검사한다. Windows는 모든 원 DELETE+READ capability를 FILE_SHARE_READ만으로 확보하여 writer/delete/rename와 배타 정책을 적용한 뒤 재검증한다. regular/reparse/singlelink 및 원 directory containment를 확인하고 SetFileInformationByHandle로 원 handle만 dispose한다. Linux는 nofollow/singlelink native pins 후 원 directory FD의 알려진 이름에 unlinkat을 사용한다. 각 mutation 전 원 directory/file identity와 남은 exact entries를 다시 확인한다. 마지막 디렉토리는 원 pin identity와 empty namespace를 확인한 뒤 제거한다. recursive 삭제나 불명확한 경로 fallback은 없다.

## 인수 조건

inactive exactset 제거/source 보존, active 거절, foreign/hash 실패 원파일 보존, Unix symlink 외부 보존, Windows metadata held handle로 DELETE preflight 실패시 전체 보존 회귀를 작성했다. active DB는 변경하지 않는다. 외부 설치·사용자 원본을 삭제하지 않는다.

## 검증과 한계

담당 source 개별 rustfmt 성공. Cargo/native는 root 검증을 기다린다. 사전 실패는 삭제0이지만 mutation 시작 후 I/O 실패는 부분 삭제를 남길 수 있어 원자성을 주장하지 않는다. 남은 파일은 보존하고 재귀 정리를 하지 않는다. Linux native named unlink에는 검사와 syscall 사이 namespace 교체 경합이 있어 외부 concurrent attacker를 완전히 배제한다고 주장하지 않는다. Windows 원 DELETE handles는 실제 native preflight 회귀가 통과해야 채택 근거가 된다. root shared source/Cargo/schema는 변경하지 않았다.
