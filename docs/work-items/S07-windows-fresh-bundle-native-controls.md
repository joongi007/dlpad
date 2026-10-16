# S07: 새 설치 묶음의 실제 취소·기한 보존

- 상태: done
- 분류: verification
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — 공유 계약·통합·Windows native 검증; diagnostic_cache — /tmp 전용 C# 원 프로세스 제어 observer와 Python collector 초안. 프로젝트 게시 파일은 scripts/windows_user_bundle_control_probe.cs 및 scripts/windows_user_bundle_control_collect.py 후보이며 root가 조정한다.
- 선행 작업: S07-windows-fresh-bundle-coordinator 및 S07-windows-fresh-bundle-cli 전체 검사

## 사용자 결과

새 설치 복사 중 취소 또는 기한 만료를 완료로 표시하지 않고 부분 파일과 외부 파일을 보존한다.

## 계약과 범위

기존 원 프로세스 HANDLE·생성 시각·소유 Job·원 primary thread·전용 console group 계약을 재사용한다. 사용자 데스크톱을 사용하지 않는다. 원 복사 prefix 관측 이후만 제어한다. 기한은 충분한 시작 예산을 주고 관측한 원 thread를 그 정확 기한 이후까지 bounded suspension한 합성 scheduling 조건으로 시험한다. 실물 디스크 정지·전원손실 인수로 표현하지 않는다. 기존 제품 취소·기한 정책과 Cargo/schema를 변경하지 않는다.

## 인수 조건

실제 copied prefix 관측, 정상 원 Job 종료, Interrupted/TimedOut 및 mutation true, 원 파일 prefix/native ID/source/sentinel 보존을 대조한다. 복사 전 타임아웃이나 parser 거절을 복사 중 시험 성공으로 대체하지 않는다. 강제 정리는 FAIL이며 retained capability 관측은 종료 전 상태이고 종료 후 권한이 아니다.

## 검증과 handoff

현재 /tmp 초안 준비 단계이며 실제 제어 시험은 미실행이다. root가 전체 검증 완료 후 독립 검토와 실제 격리 Windows 시험을 진행한다.

전용C#/Python초안을 exactSHA로 root가 scripts/windows_user_bundle_control_probe.cs 및 windows_user_bundle_control_collect.py에통합했다. 원privateconsole/exact상속HANDLE2/원process·thread·Job·30초deadline/복사prefix 이후 bounded suspend를사용한다. compile raw terminal·exactwholeproof·GNU/lock/inputpins·source전후·stdoutreader boundeddrain/fault도검증한다. --help·AST는통과했고 실제C#compile/취소/기한은아직미실행이다. 원RootId helper는volume32/fileindex64이므로128bitnativeID증명으로확대하지않는다. Pythonhelper 원capability private경로와원source구현은별도검토했다.

실제Windows compile+cancel+timeout3명령이정상종료했고 root검토PASS다. cancel원복사prefix2,818,048B·timeout4,390,912B를원128MiB합성PE와stream대조했다. timeout은원primarythread30,324ms 합성scheduling hold이후30초기한에서TimedOut이며실물I/O stall이아니다. 두원Job자연0·exit1/Interrupted또는TimedOut·opaque원cap보유true·stdoutdraintrue/faultfalse·source/manifest/notice/foreign보존·ready/receipt/active미게시를확인했다. root-review.json은evidence/windows-msvc-fresh-bundle-copy-controls-20261006-continue에저장했고독립최종검토를기다린다.

범위 인수 완료: capability교정 source579의 전체4검사·774/0/13 및 실제 fresh CLI14명령/11행·GUI3case·원NativeID10, 별도 복사중control3명령/2case를 root/독립검토했다. 밝은테마PNG와읽기전용SQLite Light/integrity·원source/등록/receipt/active/foreign 보존을확인했다. 최초parse/시험rename/collectorcasekey FAIL은보존한다. 이source579의정확map 인수이며 이후wake계측통합 source579는같은항목수지만별도검증중이다. durablejournal·upgrade/rollback·기본설치위치/UI·cleanOS/서명·전체P0 완료는범위밖후속이다.
