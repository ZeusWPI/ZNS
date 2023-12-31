:- use_module('dns').
:- use_module('notifier').
:- use_module('worker').
:- use_module('config').
:- use_module('log').

main() :-
    print_info,
    read_config,

    repeat,
        udp_socket(Socket),
        tcp_bind(Socket, ip(127,0,0,1):53),
        log_info('Accesible on IP: ~s PORT: ~d', ['127.0.0.1', 53]),
        message_queue_create(JobQueue, [max_size(1024)]),
        message_queue_create(NotifyQueue, [max_size(1024)]),

        thread_create(server(Socket, JobQueue), ServerId, []),
        thread_create(notifier:run(NotifyQueue), NotifierId, []),
        spawn_workers(4, Socket, JobQueue, NotifyQueue, WorkerIds),

        thread_join(ServerId, _),
        thread_join(NotifierId, _),
        foreach(member(WorkerId, WorkerIds), thread_join(WorkerId, _)),
        fail.

spawn_workers(0, _     , _    , _          , []).
spawn_workers(N, Socket, JobQueue, NotifyQueue, [WorkerId | WorkerIds]) :-
    NM is N - 1,
    thread_create(worker:run(Socket, JobQueue, NotifyQueue), WorkerId, []),
    spawn_workers(NM, Socket, JobQueue, NotifyQueue, WorkerIds).


server(Socket, JobQueue) :-
    log_debug('Ingress Worker ready to accept requests'),
    repeat,
        udp_receive(Socket, Data, From, [as(codes)]),
        thread_send_message(JobQueue, packet(From, Data)),
        fail.

print_info :-
    current_prolog_flag(version_data, swi(Ma, Mi, Pa, _)),
    
    log_info('╭────────────────────────────────────────────────╮'),
    log_info('│ ZNS - THE ZEUS (DOMAIN) NAME (SYSTEM) SERVER   │'),
    log_info('│ Running on SWI-Prolog: ~d.~d.~d                  │ ', [Ma, Mi, Pa]),
    log_info('╰────────────────────────────────────────────────╯').