OUTPUT=/code/118010469/output
EOUTPUT=/code/118010469/error
EXEC=/code/118010469/nbody
function gen_mpi_script() {
    tempfile=$(mktemp /tmp/118010469.XXXXXXXXXXXXXXXXXXXXXXX.pbs)
    out=$(mktemp $OUTPUT/118010469.XXXXXXXXXXXXXXXXXXXXXXX.out)
    err=$(mktemp $EOUTPUT/118010469.XXXXXXXXXXXXXXXXXXXXXXX.err)
    cat <<EOF > $tempfile
#!/bin/bash
#PBS -l nodes=1:ppn=5,mem=1g,walltime=00:02:00
#PBS -q batch
#PBS -m abe
#PBS -V
#PBS -o $out
#PBS -e $err
timeout 10 mpiexec -n $1 $EXEC -n $2 -m benchmark -e mpi_normal
EOF
    chmod 777 $tempfile
    echo $tempfile
}

function gen_mpip_script() {
    tempfile=$(mktemp /tmp/118010469.XXXXXXXXXXXXXXXXXXXXXXX.pbs)
    out=$(mktemp $OUTPUT/118010469.XXXXXXXXXXXXXXXXXXXXXXX.out)
    err=$(mktemp $EOUTPUT/118010469.XXXXXXXXXXXXXXXXXXXXXXX.err)
    cat <<EOF > $tempfile
#!/bin/bash
#PBS -l nodes=1:ppn=5,mem=1g,walltime=00:02:00
#PBS -q batch
#PBS -m abe
#PBS -V
#PBS -o $out
#PBS -e $EOUTPUT
timeout 60 mpiexec -n $1 $EXEC -n $2 -t $3 -m benchmark -e mpi_pthread
EOF
    chmod 777 $tempfile
    echo $tempfile
}

function gen_thread_script() {
    tempfile=$(mktemp /tmp/118010469.XXXXXXXXXXXXXXXXXXXXXXX.pbs)
    out=$(mktemp $OUTPUT/118010469.XXXXXXXXXXXXXXXXXXXXXXX.out)
    err=$(mktemp $EOUTPUT/118010469.XXXXXXXXXXXXXXXXXXXXXXX.err)
    cat <<EOF > $tempfile
#!/bin/bash
#PBS -l nodes=1:ppn=5,mem=1g,walltime=00:02:00
#PBS -q batch
#PBS -m abe
#PBS -V
#PBS -o $out
#PBS -e $err
timeout 10 $EXEC -n $2 -e tree -m benchmark
timeout 10 $EXEC -n $2 -e brute_force -m benchmark
timeout 10 $EXEC -t $1 -n $2 -e pthread -m benchmark
timeout 10 $EXEC -t $1 -n $2 -e openmp -m benchmark
timeout 10 $EXEC -t $1 -n $2 -e rayon -m benchmark
timeout 10 $EXEC -t $1 -n $2 -e rayon_tree -m benchmark
EOF
   chmod 777 $tempfile
   echo $tempfile
}

function gen_seq_script() {
    tempfile=$(mktemp /tmp/118010469.XXXXXXXXXXXXXXXXXXXXXXX.pbs)
    out=$(mktemp $OUTPUT/118010469.XXXXXXXXXXXXXXXXXXXXXXX.out)
    err=$(mktemp $EOUTPUT/118010469.XXXXXXXXXXXXXXXXXXXXXXX.err)
    cat <<EOF > $tempfile
#!/bin/bash
#PBS -l nodes=1:ppn=5,mem=1g,walltime=00:02:00
#PBS -q batch
#PBS -m abe
#PBS -V
#PBS -o $out
#PBS -e $err
timeout 10 $EXEC -n $1 -e tree -m benchmark
timeout 10 $EXEC -n $1 -e brute_force -m benchmark
EOF
   chmod 777 $tempfile
   echo $tempfile
}

function auto_submit_seq() {
    for j in $(seq 1000 1000 20000)
    do 
        script=$(gen_seq_script j)
        qsub $script
        echo "submit seq $j"
    done
}

function auto_submit_mpi() {
        for i in 1 2 3 4 5 6 7 8
    do
        for j in $(seq 1000 1000 20000)
        do 
            script=$(gen_mpi_script i j)
            qsub $script
            echo "submit mpi $j/$i"
        done
    done
}

function auto_submit_thd() {
        for i in 1 2 3 4 5 6 7 8
    do
        for j in $(seq 1000 1000 20000)
        do 
            script=$(gen_thread_script i j)
            qsub $script
            echo "submit mpi $j/$i"
        done
    done
}
