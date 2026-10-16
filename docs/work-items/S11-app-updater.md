# S11: 기능 완성과 Git·CI/CD 이후 앱 updater

- 상태: proposed
- 분류: feature
- 우선순위: P1
- 요구사항: R05, R08, R10, R11
- 담당/수정 범위: 후속 통합 담당; 앱 updater 구현 파일은 선행 단계 완료 후 확정
- 선행 작업: 사용자 요청 기능 완성·검증, 정상 Git 및 사용자 저장소에 반영, CI/CD 연결·실제 성공

## 사용자 결과

2026-10-03 사용자가 기능을 모두 만든 뒤 Git과 CI/CD에 올리고, 그 이후 앱 updater를 만들도록 순서를 지정했다. 이 작업은 앱 자체 업데이트이며 외부 사용자 도구 설치를 임의로 덮어쓰는 권한이 아니다.

## 계약과 범위

현재는 후속 순서만 기록한다. 앞 단계의 버그 수정·실패 경로·OS 회귀 검증을 우선한다. 업데이트 채널·서명 metadata·배포 대상·OS별 교체/rollback 정책은 실제 패키지/CI 결과를 근거로 구체화한다. 기존 .git을 삭제/재구성하거나 저장소 주소·공개 배포를 추측하지 않는다.

## 인수 조건

선행 기능 인수와 Git/CI/CD 실제 성공을 확인한 뒤 구현 착수. 정상 업데이트·손상/서명 오류 거절·중단/설치 실패 복구·기존 버전 유지·사용자 미디어/외부 도구 보존을 OS별 실제 시험으로 확인한다. 구체적 배포·권한 조건은 구현 전에 기록한다.

## 검증과 handoff

사용자 후속 요청을 기록했으며 updater 구현/배포/성공 검증은 아직 수행하지 않았다. 현재 정상 Git 저장소가 없고 P0·전체 기능 게이트가 미완료이므로 선행 작업 완료 전이다.

2026-10-11 준비 범위: scripts/validate_update_candidate.py는 최대64KiB의 중복 없는 JSON, 새 버전/channel/WindowsMSVC target, allowlist HTTPS host와 credential/query/fragment 금지, SHA256·256MiB package 상한 및 unsigned release_ready=false를 검사한다. root 정책 회귀3개 통과. 이는 다운로드·서명 검증·실제 교체·rollback 구현이 아니다. 별도 source Git bundle 도구와 Windows 우선 CI 구성을 준비했으며 외부 저장소 URL·실제 Actions 성공은 남는다.
