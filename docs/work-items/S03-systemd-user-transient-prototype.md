# S03: 사용자 systemd transient guardian 합성 실험

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R08
- 담당/수정 범위: soak_recovery_audit; scripts/systemd_user_guardian_probe.py, 이 workitem, docs/validation/systemd-user-transient-prototype.md, 전용 evidence/systemd-user-transient-20261003/
- 선행 작업: S03-systemd-user-readiness, S03-app-subreaper-fallback-review

## 사용자 결과

guardian 고장 보완 후보의 사용자 권한·FD·소유cgroup 정리와 literal argv 경계를 실제 합성 자료로 확인한다. Proposed 선택이며 제품 채택/구현이 아니다.

## 계약과 범위

README→PRODUCT→CONTEXT→R08→ADR-0002/ARCHITECTURE를 읽었다. 고유 dlpad-fixture UUID transient 사용자 service와 새 /tmp 합성 guardian/setsid 자손만 사용한다. 정확unit/InvocationID/MainPID/start/cgroup 확인 후 소유 fixture만 종료한다. 관리자/global 설정/GUI/기존8hPID·unit/제품/Cargo/schema 변경은 없다. 하위 agent를 생성하지 않는다.

## 인수 조건

stdin Unixsocket FD0 직접전달/readiness와EOF, guardianSIGKILL 후 전체cgroup 정리, 무관한소유sentinel 생존, raw `$VAR`/`${VAR}` literalargv를 실제검증한다. 로그/result bounded durable 저장, exactidentity/cleanup을 기록하며 argv drift는 실패로 보존한다.

## 검증과 handoff

scripts/systemd_user_guardian_probe.py 실제2case실험exit2: FD0 Unixsocket/EOF/guardianSIGKILL 뒤setsid+leaf 전체cgroup정리와sentinel보존 capability PASS, rawliteralargv FAIL. EOF0/7.39ms, SIGKILL255/6.28ms. 정확6identity부재/pidfdready/cgroupremoved/tmpcleanup. 원시result/log보존. 연구작업만완료이며제품채택/R08완료아님. ExecStartEx/no-env-expand후속을별도scope로진행한다.
